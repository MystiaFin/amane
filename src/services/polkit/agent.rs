use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{LazyLock, Mutex, PoisonError};
use std::thread;
use std::time::Duration;

use zbus::blocking::{Connection, MessageIterator, Proxy, connection::Builder};
use zbus::message::Header;
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

use super::{
    Polkit, PolkitConfig, PolkitError, PolkitFlow, PolkitIdentity, PolkitMessage, PolkitResult,
    native,
    state::{Pending, State},
};
use crate::Service;

const AUTHORITY: &str = "org.freedesktop.PolicyKit1";
const AUTHORITY_PATH: &str = "/org/freedesktop/PolicyKit1/Authority";
const AUTHORITY_INTERFACE: &str = "org.freedesktop.PolicyKit1.Authority";
const METHOD_TIMEOUT: Duration = Duration::from_secs(5);
pub(super) type WireIdentity = (String, HashMap<String, OwnedValue>);
pub(super) type Subject = (String, HashMap<String, OwnedValue>);

#[derive(Debug, zbus::DBusError)]
#[zbus(prefix = "org.freedesktop.PolicyKit1.Error")]
pub(super) enum AgentError {
    Cancelled(String),
    Failed(String),
    NotAuthorized(String),
    #[zbus(error)]
    ZBus(zbus::Error),
}

pub(super) type Reply = Result<(), AgentError>;

pub(super) enum Command {
    Start(PolkitConfig),
    Stop,
    Select(u64, usize),
    Respond(u64, u64, super::native::Secret),
    Retry(u64),
    Cancel(u64),
    Begin(u64, Box<Pending>),
    AuthorityCancel(u64, String),
    Lost(u64, String),
    Native(u64, NativeEvent),
}

pub(super) enum NativeEvent {
    Prompt(String, bool),
    Message(String, bool),
    Completed(bool),
}

struct Mailbox {
    sender: Sender<Command>,
    receiver: Mutex<Option<Receiver<Command>>>,
}
static MAILBOX: LazyLock<Mailbox> = LazyLock::new(|| {
    let (sender, receiver) = mpsc::channel();
    Mailbox {
        sender,
        receiver: Mutex::new(Some(receiver)),
    }
});

pub(super) fn send(command: Command) -> Result<(), PolkitError> {
    // Ensure the listener exists, without taking a second read lock in an input handler.
    crate::services::store::find::<Polkit>();
    MAILBOX
        .sender
        .send(command)
        .map_err(|_| PolkitError::Unavailable)
}

pub(super) fn run() {
    let Some(receiver) = MAILBOX
        .receiver
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take()
    else {
        return;
    };
    let context = match native::Context::new() {
        Ok(context) => context,
        Err(error) => {
            Polkit::write().error = Some(error);
            return;
        }
    };
    let mut runtime = Runtime::new(MAILBOX.sender.clone());
    loop {
        if runtime.session.is_some() {
            context.dispatch();
        }
        let command = if runtime.session.is_some() {
            match receiver.recv_timeout(Duration::from_millis(10)) {
                Ok(command) => command,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        } else {
            match receiver.recv() {
                Ok(command) => command,
                Err(_) => break,
            }
        };
        runtime.handle(command);
        *Polkit::write() = runtime.state.view.clone();
    }
}

struct Agent {
    epoch: u64,
    owner: String,
    sender: Sender<Command>,
    next_id: AtomicU64,
}

impl Agent {
    fn authenticate_sender(&self, header: &Header<'_>) -> Reply {
        if header
            .sender()
            .is_some_and(|sender| sender.as_str() == self.owner)
        {
            return Ok(());
        }
        Err(AgentError::NotAuthorized(
            "only the registered Polkit authority may call this agent".into(),
        ))
    }
}

#[zbus::interface(name = "org.freedesktop.PolicyKit1.AuthenticationAgent")]
impl Agent {
    #[expect(
        clippy::too_many_arguments,
        reason = "the Polkit wire signature and authenticated message header"
    )]
    async fn begin_authentication(
        &self,
        action_id: String,
        message: String,
        icon_name: String,
        details: BTreeMap<String, String>,
        cookie: String,
        identities: Vec<WireIdentity>,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<(), AgentError> {
        self.authenticate_sender(&header)?;
        let identities = read_identities(identities)?;
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (reply, receiver) = async_channel::bounded(1);
        let pending = Pending {
            flow: PolkitFlow::new(id, action_id, message, icon_name, details, identities),
            cookie,
            reply,
        };
        self.sender
            .send(Command::Begin(self.epoch, Box::new(pending)))
            .map_err(|_| AgentError::Failed("authentication service stopped".into()))?;
        receiver
            .recv()
            .await
            .map_err(|_| AgentError::Cancelled("authentication service stopped".into()))?
    }

    fn cancel_authentication(
        &self,
        cookie: String,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<(), AgentError> {
        self.authenticate_sender(&header)?;
        self.sender
            .send(Command::AuthorityCancel(self.epoch, cookie))
            .map_err(|_| AgentError::Failed("authentication service stopped".into()))
    }
}

pub(super) fn read_identities(
    identities: Vec<WireIdentity>,
) -> Result<Vec<PolkitIdentity>, AgentError> {
    if identities.is_empty() {
        return Err(AgentError::Failed(
            "no authentication identities were supplied".into(),
        ));
    }
    identities
        .into_iter()
        .map(|(kind, fields)| {
            // PolkitAgentSession only authenticates Unix users. Reject unsupported wire identities explicitly.
            if kind != "unix-user" {
                return Err(AgentError::Failed(format!(
                    "unsupported authentication identity: {kind}"
                )));
            }
            let uid = fields
                .get("uid")
                .and_then(|value| u32::try_from(value).ok())
                .filter(|uid| *uid <= i32::MAX as u32)
                .ok_or_else(|| AgentError::Failed("invalid Unix user identity".into()))?;
            Ok(PolkitIdentity {
                uid,
                name: uid.to_string(),
            })
        })
        .collect()
}

pub(super) struct Registration {
    pub connection: Connection,
    subject: Subject,
    path: String,
    owner: String,
    watcher: Option<thread::JoinHandle<()>>,
}

impl Registration {
    pub fn new(
        builder: Builder<'_>,
        config: &PolkitConfig,
        epoch: u64,
        sender: Sender<Command>,
    ) -> Result<Self, PolkitError> {
        builder
            .method_timeout(METHOD_TIMEOUT)
            .build()
            .and_then(|connection| Self::connect(connection, config, epoch, sender))
            .map_err(|error| PolkitError::Registration(error.to_string()))
    }

    fn connect(
        connection: Connection,
        config: &PolkitConfig,
        epoch: u64,
        sender: Sender<Command>,
    ) -> zbus::Result<Self> {
        let rule = zbus::MatchRule::builder()
            .msg_type(zbus::message::Type::Signal)
            .sender("org.freedesktop.DBus")?
            .interface("org.freedesktop.DBus")?
            .member("NameOwnerChanged")?
            .add_arg(AUTHORITY)?
            .build();
        let events = MessageIterator::for_match_rule(rule, &connection, Some(8))?;
        // Ping allows D-Bus to activate the authority before resolving its unique sender.
        connection.call_method(
            Some(AUTHORITY),
            AUTHORITY_PATH,
            Some("org.freedesktop.DBus.Peer"),
            "Ping",
            &(),
        )?;
        let owner: String = connection
            .call_method(
                Some("org.freedesktop.DBus"),
                "/org/freedesktop/DBus",
                Some("org.freedesktop.DBus"),
                "GetNameOwner",
                &(AUTHORITY,),
            )?
            .body()
            .deserialize()?;
        let session: OwnedObjectPath = connection
            .call_method(
                Some("org.freedesktop.login1"),
                "/org/freedesktop/login1",
                Some("org.freedesktop.login1.Manager"),
                "GetSessionByPID",
                &(std::process::id(),),
            )?
            .body()
            .deserialize()?;
        let session_proxy = Proxy::new(
            &connection,
            "org.freedesktop.login1",
            session.as_str(),
            "org.freedesktop.login1.Session",
        )?;
        let id: String = session_proxy.get_property("Id")?;
        let subject = (
            "unix-session".into(),
            HashMap::from([(
                "session-id".into(),
                OwnedValue::from(zbus::zvariant::Str::from(id)),
            )]),
        );
        connection.object_server().at(
            config.path.as_str(),
            Agent {
                epoch,
                owner: owner.clone(),
                sender: sender.clone(),
                next_id: AtomicU64::new(epoch << 32),
            },
        )?;
        let mut registration = Self {
            connection,
            subject,
            path: config.path.clone(),
            owner: owner.clone(),
            watcher: None,
        };
        registration.connection.call_method(
            Some(owner.as_str()),
            AUTHORITY_PATH,
            Some(AUTHORITY_INTERFACE),
            "RegisterAuthenticationAgent",
            &(&registration.subject, &config.locale, &registration.path),
        )?;
        // Keep the subscription's race window covered: an owner change after resolution invalidates this registration.
        registration.watcher = Some(thread::spawn(move || {
            for event in events {
                let Ok(event) = event else {
                    break;
                };
                if let Ok((_, old, new)) = event.body().deserialize::<(String, String, String)>()
                    && old == owner
                    && new != owner
                {
                    let _ = sender.send(Command::Lost(
                        epoch,
                        "the Polkit authority restarted".into(),
                    ));
                    return;
                }
            }
            let _ = sender.send(Command::Lost(epoch, "the system bus disconnected".into()));
        }));
        Ok(registration)
    }

    pub fn unregister(&self) -> Result<(), PolkitError> {
        self.connection
            .call_method(
                Some(self.owner.as_str()),
                AUTHORITY_PATH,
                Some(AUTHORITY_INTERFACE),
                "UnregisterAuthenticationAgent",
                &(&self.subject, &self.path),
            )
            .map(|_| ())
            .map_err(|error| PolkitError::Registration(error.to_string()))
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        let _ = self.connection.clone().close();
        if let Some(watcher) = self.watcher.take() {
            let _ = watcher.join();
        }
    }
}

struct Runtime {
    state: State,
    registration: Option<Registration>,
    session: Option<native::Session>,
    epoch: u64,
    sender: Sender<Command>,
}

impl Runtime {
    fn new(sender: Sender<Command>) -> Self {
        Self {
            state: State::new(),
            registration: None,
            session: None,
            epoch: 0,
            sender,
        }
    }

    fn handle(&mut self, command: Command) {
        match command {
            Command::Start(config) => {
                self.stop();
                self.epoch += 1;
                self.state.view.enabled = true;
                self.state.view.path = config.path.clone();
                self.state.view.error = None;
                let result = Builder::system()
                    .map_err(|error| PolkitError::Registration(error.to_string()))
                    .and_then(|builder| {
                        Registration::new(builder, &config, self.epoch, self.sender.clone())
                    });
                match result {
                    Ok(registration) => {
                        self.registration = Some(registration);
                        self.state.view.running = true;
                    }
                    Err(error) => {
                        self.state.view.error = Some(error);
                    }
                }
            }
            Command::Stop => self.stop(),
            Command::Begin(epoch, mut pending) => {
                if epoch != self.epoch || !self.state.view.running {
                    let _ = pending.reply.try_send(Err(AgentError::Cancelled(
                        "agent is no longer registered".into(),
                    )));
                    return;
                }
                if self.state.queue.len() >= 32
                    || self
                        .state
                        .active
                        .as_ref()
                        .is_some_and(|active| active.cookie == pending.cookie)
                    || self
                        .state
                        .queue
                        .iter()
                        .any(|active| active.cookie == pending.cookie)
                {
                    let _ = pending.reply.try_send(Err(AgentError::Failed(
                        "duplicate request or authentication queue is full".into(),
                    )));
                    return;
                }
                // Account lookup can involve NSS, so keep it off the D-Bus executor and UI thread.
                for identity in &mut pending.flow.identities {
                    identity.name = native::account_name(identity.uid);
                }
                if self.state.begin(*pending) {
                    self.start_session();
                }
            }
            Command::Respond(request, prompt, response) => {
                if self.state.take_response(request, prompt)
                    && let Some(session) = &self.session
                {
                    session.respond(&response);
                }
            }
            Command::Select(request, index) => {
                if self.state.restart(request, Some(index)) {
                    self.start_session();
                }
            }
            Command::Retry(request) => {
                if self.session.is_none() && self.state.restart(request, None) {
                    self.start_session();
                }
            }
            Command::Cancel(request) => {
                if self
                    .state
                    .active
                    .as_ref()
                    .is_some_and(|active| active.flow.id == request)
                {
                    self.session = None;
                }
                if self.state.cancel(request) {
                    self.start_session();
                } else if self.state.active.is_none() {
                    self.session = None;
                }
            }
            Command::AuthorityCancel(epoch, cookie) if epoch == self.epoch => {
                if self
                    .state
                    .active
                    .as_ref()
                    .is_some_and(|request| request.cookie == cookie)
                {
                    self.session = None;
                }
                match self.state.cancel_cookie(&cookie) {
                    Ok(true) => self.start_session(),
                    Err(error) => self.handle(Command::Lost(epoch, error.to_string())),
                    Ok(false) => {}
                }
            }
            Command::Lost(epoch, message) if epoch == self.epoch => {
                let error = PolkitError::Disconnected(message);
                self.session = None;
                self.state.cancel_all(PolkitResult::Error(error.clone()));
                self.registration = None;
                self.state.view.running = false;
                self.state.view.error = Some(error);
            }
            Command::Native(generation, event) if generation == self.state.generation => {
                match event {
                    NativeEvent::Prompt(text, visible) => {
                        self.state.prompt(generation, text, visible)
                    }
                    NativeEvent::Message(text, error) => {
                        if let Some(flow) = self.state.view.flow.as_mut() {
                            flow.supplementary = Some(PolkitMessage { text, error });
                        }
                    }
                    NativeEvent::Completed(success) => {
                        self.session = None;
                        if success {
                            if self.state.finish(PolkitResult::Success) {
                                self.start_session();
                            }
                        } else {
                            self.state.failed();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn start_session(&mut self) {
        self.session = None;
        let Some(request) = self.state.active.as_ref() else {
            return;
        };
        let Some(flow) = self.state.view.flow.as_ref() else {
            return;
        };
        match native::Session::new(
            flow.identities[flow.selected].uid,
            &request.cookie,
            self.state.generation,
            self.sender.clone(),
        ) {
            Ok(session) => {
                session.start();
                self.session = Some(session);
            }
            Err(error) => {
                if self.state.finish(PolkitResult::Error(error)) {
                    self.start_session();
                }
            }
        }
    }

    fn stop(&mut self) {
        self.session = None;
        self.state.cancel_all(PolkitResult::Cancelled);
        self.epoch += 1;
        self.state.view.enabled = false;
        self.state.view.running = false;
        if let Some(registration) = self.registration.take() {
            self.state.view.error = registration.unregister().err();
        }
    }
}
