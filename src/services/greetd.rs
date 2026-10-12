mod protocol;

use std::env;
use std::os::unix::net::UnixStream;
use std::sync::LazyLock;
use std::sync::mpsc::{self, Receiver, Sender};

use crate::Service;

use protocol::Reply;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GreetdConnection {
    #[default]
    Connecting,
    Connected,
    Disconnected,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GreetdStatus {
    #[default]
    Idle,
    Authenticating,
    Authenticated,
    Cancelling,
    AuthenticationFailed,
    StartingSession,
    SessionStarted,
    SessionFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthenticationMessageType {
    Visible,
    Secret,
    Information,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticationMessage {
    kind: AuthenticationMessageType,
    text: String,
}

impl AuthenticationMessage {
    pub fn kind(&self) -> AuthenticationMessageType {
        self.kind
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

pub struct Greetd {
    connection: GreetdConnection,
    status: GreetdStatus,
    message: Option<AuthenticationMessage>,
    error: Option<String>,
}

impl Service for Greetd {
    fn new() -> Self {
        Self {
            connection: GreetdConnection::Connecting,
            status: GreetdStatus::Idle,
            message: None,
            error: None,
        }
    }

    fn listen() {
        let receiver = COMMANDS
            .1
            .lock()
            .expect("failed to lock greetd command receiver")
            .take()
            .expect("greetd listener started twice");

        let mut stream = connect();

        for command in receiver {
            run(command, &mut stream);
        }
    }
}

impl Greetd {
    pub fn connection(&self) -> GreetdConnection {
        self.connection
    }

    pub fn connected(&self) -> bool {
        self.connection == GreetdConnection::Connected
    }

    pub fn status(&self) -> GreetdStatus {
        self.status
    }

    pub fn message(&self) -> Option<&AuthenticationMessage> {
        self.message.as_ref()
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn connect() {
        initialize();

        {
            let mut greetd = Self::write();
            greetd.connection = GreetdConnection::Connecting;
            greetd.error = None;
        }

        send(Command::Connect);
    }

    pub fn authenticate(username: impl Into<String>) {
        initialize();

        {
            let mut greetd = Self::write();
            greetd.status = GreetdStatus::Authenticating;
            greetd.message = None;
            greetd.error = None;
        }

        send(Command::Authenticate(username.into()));
    }

    pub fn respond(response: impl Into<String>) {
        answer(Some(response.into()));
    }

    // Information and error messages are acknowledged without a response.
    pub fn acknowledge() {
        answer(None);
    }

    pub fn cancel() {
        initialize();

        {
            let mut greetd = Self::write();
            greetd.status = GreetdStatus::Cancelling;
            greetd.message = None;
            greetd.error = None;
        }

        send(Command::Cancel);
    }

    pub fn start_session<C, E>(command: C, environment: E)
    where
        C: IntoIterator,
        C::Item: Into<String>,
        E: IntoIterator,
        E::Item: Into<String>,
    {
        initialize();

        {
            let mut greetd = Self::write();
            greetd.status = GreetdStatus::StartingSession;
            greetd.message = None;
            greetd.error = None;
        }

        send(Command::StartSession {
            command: command.into_iter().map(Into::into).collect(),
            environment: environment.into_iter().map(Into::into).collect(),
        });
    }
}

fn answer(response: Option<String>) {
    initialize();

    {
        let mut greetd = Greetd::write();
        greetd.message = None;
        greetd.error = None;
    }

    send(Command::Respond(response));
}

fn initialize() {
    drop(Greetd::read());
}

enum Command {
    Connect,
    Authenticate(String),
    Respond(Option<String>),
    Cancel,
    StartSession {
        command: Vec<String>,
        environment: Vec<String>,
    },
}

#[derive(Clone, Copy)]
enum Operation {
    Authenticate,
    Cancel,
    StartSession,
}

type CommandChannel = (Sender<Command>, std::sync::Mutex<Option<Receiver<Command>>>);

static COMMANDS: LazyLock<CommandChannel> = LazyLock::new(|| {
    let (sender, receiver) = mpsc::channel();

    (sender, std::sync::Mutex::new(Some(receiver)))
});

fn send(command: Command) {
    COMMANDS
        .0
        .send(command)
        .expect("failed to hand a command to the greetd service");
}

fn connect() -> Option<UnixStream> {
    let result = env::var("GREETD_SOCK")
        .map_err(|error| format!("failed to read GREETD_SOCK: {error}"))
        .and_then(|path| {
            UnixStream::connect(&path)
                .map_err(|error| format!("failed to connect to greetd at {path}: {error}"))
        });

    match result {
        Ok(stream) => {
            let mut greetd = Greetd::write();
            greetd.connection = GreetdConnection::Connected;
            greetd.error = None;

            Some(stream)
        }
        Err(error) => {
            disconnected(error);
            None
        }
    }
}

fn run(command: Command, stream: &mut Option<UnixStream>) {
    if matches!(command, Command::Connect) {
        *stream = connect();
        return;
    }

    if stream.is_none() {
        *stream = connect();
    }

    let (operation, result) = {
        let Some(socket) = stream.as_mut() else {
            Greetd::write().status = GreetdStatus::Idle;
            return;
        };

        match command {
            Command::Connect => unreachable!(),
            Command::Authenticate(username) => (
                Operation::Authenticate,
                protocol::create_session(socket, &username),
            ),
            Command::Respond(response) => (
                Operation::Authenticate,
                protocol::respond(socket, response.as_deref()),
            ),
            Command::Cancel => (Operation::Cancel, protocol::cancel(socket)),
            Command::StartSession {
                command,
                environment,
            } => (
                Operation::StartSession,
                protocol::start_session(socket, &command, &environment),
            ),
        }
    };

    match result {
        Ok(reply) => apply(operation, reply),
        Err(error) => {
            *stream = None;
            disconnected(error.to_string());
        }
    }
}

fn apply(operation: Operation, reply: Reply) {
    let mut greetd = Greetd::write();

    match reply {
        Reply::Success => {
            greetd.status = match operation {
                Operation::Authenticate => GreetdStatus::Authenticated,
                Operation::Cancel => GreetdStatus::Idle,
                Operation::StartSession => GreetdStatus::SessionStarted,
            };
            greetd.message = None;
            greetd.error = None;
        }
        Reply::Message { kind, text } => {
            let Some(kind) = message_type(&kind) else {
                greetd.status = GreetdStatus::AuthenticationFailed;
                greetd.error = Some(format!(
                    "unknown greetd authentication message type: {kind}"
                ));
                return;
            };

            greetd.status = GreetdStatus::Authenticating;
            greetd.message = Some(AuthenticationMessage { kind, text });
            greetd.error = None;
        }
        Reply::Error { kind, description } => {
            greetd.message = None;
            greetd.error = Some(description);
            greetd.status = if kind == "auth_error" {
                GreetdStatus::AuthenticationFailed
            } else if matches!(operation, Operation::StartSession) {
                GreetdStatus::SessionFailed
            } else {
                GreetdStatus::Idle
            };
        }
    }
}

fn message_type(kind: &str) -> Option<AuthenticationMessageType> {
    match kind {
        "visible" => Some(AuthenticationMessageType::Visible),
        "secret" => Some(AuthenticationMessageType::Secret),
        "info" => Some(AuthenticationMessageType::Information),
        "error" => Some(AuthenticationMessageType::Error),
        _ => None,
    }
}

fn disconnected(error: String) {
    let mut greetd = Greetd::write();
    greetd.connection = GreetdConnection::Disconnected;
    greetd.status = GreetdStatus::Idle;
    greetd.message = None;
    greetd.error = Some(error);
}
