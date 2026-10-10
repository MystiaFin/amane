use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command as Process, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use zbus::blocking::{Connection, connection::Builder};
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Str};

use super::agent::{AgentError, Command, Registration, Subject, WireIdentity};
use super::*;

struct Bus {
    child: Child,
    address: String,
}

impl Bus {
    fn new() -> Self {
        let mut child = Process::new("dbus-daemon")
            .args(["--session", "--nofork", "--print-address=1"])
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut address = String::new();
        BufReader::new(child.stdout.take().unwrap())
            .read_line(&mut address)
            .unwrap();
        Self {
            child,
            address: address.trim().into(),
        }
    }
    fn builder(&self) -> Builder<'_> {
        Builder::address(self.address.as_str()).unwrap()
    }
    fn connection(&self) -> Connection {
        self.builder().build().unwrap()
    }
}

impl Drop for Bus {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct Manager;
#[zbus::interface(name = "org.freedesktop.login1.Manager")]
impl Manager {
    #[zbus(name = "GetSessionByPID")]
    fn get_session_by_pid(&self, pid: u32) -> Result<OwnedObjectPath, zbus::fdo::Error> {
        if pid != std::process::id() {
            return Err(zbus::fdo::Error::InvalidArgs("wrong process".into()));
        }
        Ok(OwnedObjectPath::try_from("/org/freedesktop/login1/session/test").unwrap())
    }
}

struct Session;
#[zbus::interface(name = "org.freedesktop.login1.Session")]
impl Session {
    #[zbus(property)]
    fn id(&self) -> &str {
        "test-session"
    }
}

struct Authority {
    events: mpsc::Sender<(&'static str, Subject, String, String)>,
    reject: bool,
    stall_unregister: bool,
}
#[zbus::interface(name = "org.freedesktop.PolicyKit1.Authority")]
impl Authority {
    fn register_authentication_agent(
        &self,
        subject: Subject,
        locale: String,
        path: String,
    ) -> Result<(), zbus::fdo::Error> {
        self.events
            .send(("register", subject, locale, path))
            .unwrap();
        if self.reject {
            return Err(zbus::fdo::Error::Failed(
                "an agent is already registered".into(),
            ));
        }
        Ok(())
    }
    async fn unregister_authentication_agent(&self, subject: Subject, path: String) {
        self.events
            .send(("unregister", subject, String::new(), path))
            .unwrap();
        if self.stall_unregister {
            std::future::pending::<()>().await;
        }
    }
}

fn authority(
    bus: &Bus,
    reject: bool,
) -> (
    Connection,
    mpsc::Receiver<(&'static str, Subject, String, String)>,
) {
    authority_with_stalled_unregister(bus, reject, false)
}

fn authority_with_stalled_unregister(
    bus: &Bus,
    reject: bool,
    stall_unregister: bool,
) -> (
    Connection,
    mpsc::Receiver<(&'static str, Subject, String, String)>,
) {
    let connection = bus.connection();
    let (events, receiver) = mpsc::channel();
    connection
        .object_server()
        .at(
            "/org/freedesktop/PolicyKit1/Authority",
            Authority {
                events,
                reject,
                stall_unregister,
            },
        )
        .unwrap();
    connection
        .object_server()
        .at("/org/freedesktop/login1", Manager)
        .unwrap();
    connection
        .object_server()
        .at("/org/freedesktop/login1/session/test", Session)
        .unwrap();
    connection
        .request_name("org.freedesktop.PolicyKit1")
        .unwrap();
    connection.request_name("org.freedesktop.login1").unwrap();
    (connection, receiver)
}

fn identities() -> Vec<WireIdentity> {
    vec![(
        "unix-user".into(),
        HashMap::from([("uid".into(), OwnedValue::from(0_u32))]),
    )]
}

fn begin(connection: &Connection, destination: &str) -> zbus::Result<zbus::Message> {
    connection.call_method(
        Some(destination),
        "/org/amane/Polkit",
        Some("org.freedesktop.PolicyKit1.AuthenticationAgent"),
        "BeginAuthentication",
        &(
            "org.example.manage",
            "Please authorize",
            "dialog-password",
            HashMap::from([("device", "disk")]),
            "test-cookie",
            identities(),
        ),
    )
}

#[test]
fn registers_and_unregisters_the_process_login_session_with_the_requested_locale_and_path() {
    let bus = Bus::new();
    let (_authority, events) = authority(&bus, false);
    let (sender, _commands) = mpsc::channel();
    let config = PolkitConfig::new()
        .locale("de_DE.UTF-8")
        .path("/org/example/Agent");
    let registration = Registration::new(bus.builder(), &config, 3, sender).unwrap();
    let (kind, subject, locale, path) = events.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(kind, "register");
    assert_eq!(subject.0, "unix-session");
    assert_eq!(
        <&str>::try_from(subject.1.get("session-id").unwrap()).unwrap(),
        "test-session"
    );
    assert_eq!(locale, "de_DE.UTF-8");
    assert_eq!(path, "/org/example/Agent");
    registration.unregister().unwrap();
    assert_eq!(
        events.recv_timeout(Duration::from_secs(5)).unwrap().0,
        "unregister"
    );
}

#[test]
fn duplicate_agent_registration_reports_the_authority_error() {
    let bus = Bus::new();
    let (_authority, _events) = authority(&bus, true);
    let (sender, _commands) = mpsc::channel();
    let error = Registration::new(bus.builder(), &PolkitConfig::new(), 1, sender)
        .err()
        .unwrap();
    assert!(
        matches!(&error, PolkitError::Registration(message) if message.contains("already registered"))
    );
}

#[test]
fn only_the_registered_authority_can_begin_or_cancel_authentication() {
    let bus = Bus::new();
    let (_authority, _events) = authority(&bus, false);
    let (sender, commands) = mpsc::channel();
    let registration = Registration::new(bus.builder(), &PolkitConfig::new(), 1, sender).unwrap();
    let destination = registration.connection.unique_name().unwrap().to_string();
    let attacker = bus.connection();
    let error = begin(&attacker, &destination).unwrap_err();
    assert!(
        matches!(error, zbus::Error::MethodError(name, _, _) if name.as_str() == "org.freedesktop.PolicyKit1.Error.NotAuthorized")
    );
    let error = attacker
        .call_method(
            Some(destination.as_str()),
            "/org/amane/Polkit",
            Some("org.freedesktop.PolicyKit1.AuthenticationAgent"),
            "CancelAuthentication",
            &("test-cookie",),
        )
        .unwrap_err();
    assert!(
        matches!(error, zbus::Error::MethodError(name, _, _) if name.as_str() == "org.freedesktop.PolicyKit1.Error.NotAuthorized")
    );
    assert!(commands.try_recv().is_err());
}

#[test]
fn authority_cancellation_remains_callable_while_begin_authentication_is_pending() {
    let bus = Bus::new();
    let (authority, _events) = authority(&bus, false);
    let (sender, commands) = mpsc::channel();
    let registration = Registration::new(bus.builder(), &PolkitConfig::new(), 7, sender).unwrap();
    let destination = registration.connection.unique_name().unwrap().to_string();
    let caller = authority.clone();
    let target = destination.clone();
    let waiting = thread::spawn(move || begin(&caller, &target));
    let Command::Begin(epoch, pending) = commands.recv_timeout(Duration::from_secs(5)).unwrap()
    else {
        panic!("expected a request");
    };
    assert_eq!(epoch, 7);
    assert_eq!(pending.flow.action_id(), "org.example.manage");
    assert_eq!(pending.flow.message(), "Please authorize");
    assert_eq!(pending.flow.icon(), "dialog-password");
    assert_eq!(pending.flow.details().get("device").unwrap(), "disk");
    assert_eq!(pending.flow.identities()[0].uid(), 0);
    let mut state = state::State::new();
    let request = pending.flow.id();
    state.begin(*pending);
    authority
        .call_method(
            Some(destination.as_str()),
            "/org/amane/Polkit",
            Some("org.freedesktop.PolicyKit1.AuthenticationAgent"),
            "CancelAuthentication",
            &("test-cookie",),
        )
        .unwrap();
    let Command::AuthorityCancel(epoch, cookie) =
        commands.recv_timeout(Duration::from_secs(5)).unwrap()
    else {
        panic!("expected cancellation");
    };
    assert_eq!(epoch, 7);
    assert_eq!(cookie, "test-cookie");
    state.cancel(request);
    let error = waiting.join().unwrap().unwrap_err();
    assert!(
        matches!(error, zbus::Error::MethodError(name, _, _) if name.as_str() == "org.freedesktop.PolicyKit1.Error.Cancelled")
    );
}

#[test]
fn completion_returns_the_pending_dbus_call_without_exposing_helper_cookies() {
    let bus = Bus::new();
    let (authority, _events) = authority(&bus, false);
    let (sender, commands) = mpsc::channel();
    let registration = Registration::new(bus.builder(), &PolkitConfig::new(), 5, sender).unwrap();
    let destination = registration.connection.unique_name().unwrap().to_string();
    let waiting = thread::spawn(move || begin(&authority, &destination));
    let Command::Begin(_, pending) = commands.recv_timeout(Duration::from_secs(5)).unwrap() else {
        panic!("expected a request");
    };
    let mut state = state::State::new();
    state.begin(*pending);
    state.finish(PolkitResult::Success);
    waiting
        .join()
        .unwrap()
        .unwrap()
        .body()
        .deserialize::<()>()
        .unwrap();
}

#[test]
fn authority_restart_invalidates_the_registration() {
    let bus = Bus::new();
    let (authority, _events) = authority(&bus, false);
    let (sender, commands) = mpsc::channel();
    let _registration = Registration::new(bus.builder(), &PolkitConfig::new(), 9, sender).unwrap();
    authority
        .release_name("org.freedesktop.PolicyKit1")
        .unwrap();
    assert!(matches!(
        commands.recv_timeout(Duration::from_secs(5)).unwrap(),
        Command::Lost(9, _)
    ));
}

#[test]
fn bus_disconnection_invalidates_the_registration() {
    let mut bus = Bus::new();
    let (_authority, _events) = authority(&bus, false);
    let (sender, commands) = mpsc::channel();
    let _registration = Registration::new(bus.builder(), &PolkitConfig::new(), 11, sender).unwrap();
    bus.child.kill().unwrap();
    bus.child.wait().unwrap();
    assert!(matches!(
        commands.recv_timeout(Duration::from_secs(5)).unwrap(),
        Command::Lost(11, _)
    ));
}

#[test]
fn an_unresponsive_authority_cannot_block_unregister_indefinitely() {
    let bus = Bus::new();
    let (_authority, _events) = authority_with_stalled_unregister(&bus, false, true);
    let (sender, _commands) = mpsc::channel();
    let registration = Registration::new(bus.builder(), &PolkitConfig::new(), 13, sender).unwrap();
    let (reply, result) = mpsc::channel();
    let waiting = thread::spawn(move || {
        let error = registration.unregister().unwrap_err();
        drop(registration);
        reply.send(error).unwrap();
    });
    let error = result
        .recv_timeout(Duration::from_secs(7))
        .expect("unregister must time out");
    assert!(matches!(error, PolkitError::Registration(_)));
    waiting.join().unwrap();
}

#[test]
fn malformed_or_unsupported_identities_are_rejected_instead_of_authenticating_another_user() {
    for identity in [
        vec![],
        vec![(
            "unix-group".into(),
            HashMap::from([("gid".into(), OwnedValue::from(0_u32))]),
        )],
        vec![("unix-user".into(), HashMap::new())],
        vec![(
            "unix-user".into(),
            HashMap::from([("uid".into(), OwnedValue::from(Str::from("0")))]),
        )],
    ] {
        assert!(matches!(
            agent::read_identities(identity),
            Err(AgentError::Failed(_))
        ));
    }
}

#[test]
fn invalid_configuration_and_response_protocol_bytes_are_rejected() {
    assert!(
        PolkitConfig::new()
            .path("relative/path")
            .validate()
            .is_err()
    );
    assert!(PolkitConfig::new().locale("en\0US").validate().is_err());
    for response in ["a\0b", "a\nb", "a\rb"] {
        assert!(matches!(
            native::Secret::new(response),
            Err(PolkitError::InvalidResponse)
        ));
    }
    assert!(native::Secret::new("").is_ok());
    assert!(native::Secret::new("pässwörd").is_ok());
}
