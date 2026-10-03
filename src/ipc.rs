use std::collections::HashMap;
use std::env;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::PathBuf;

pub struct IpcCall {
    pub(crate) name: String,

    pub(crate) arguments: Vec<String>,
}

type Handler = fn(&[String]) -> String;

#[derive(Default)]
pub struct IpcHandlers {
    pub by_name: HashMap<String, Handler>,
}

impl IpcCall {
    pub fn new(name: &str, arguments: &[String]) -> Self {
        Self {
            name: String::from(name),
            arguments: arguments.to_vec(),
        }
    }

    // the name goes on the first line, then each argument on its own line
    pub fn write(&self, writer: &mut impl Write) -> io::Result<()> {
        writeln!(writer, "{}", self.name)?;

        for argument in &self.arguments {
            writeln!(writer, "{argument}")?;
        }

        Ok(())
    }

    // reads until the sender closes its side, which marks the end of the call
    pub(crate) fn read(reader: impl Read) -> io::Result<Self> {
        let mut lines = BufReader::new(reader).lines();

        let Some(name) = lines.next() else {
            return Err(io::ErrorKind::UnexpectedEof.into());
        };

        let name = name?;

        let arguments = lines.collect::<io::Result<Vec<String>>>()?;

        Ok(Self { name, arguments })
    }
}

impl IpcHandlers {
    pub fn insert(&mut self, name: &str, handler: Handler) {
        self.by_name.insert(String::from(name), handler);
    }

    // the reply is whatever the handler returns, so an unknown name answers in words too
    pub fn run(&self, call: &IpcCall) -> String {
        let Some(handler) = self.by_name.get(&call.name) else {
            return format!("no handler named {}", call.name);
        };

        handler(&call.arguments)
    }
}

pub fn ipc_socket() -> PathBuf {
    // every wayland session sets this, and it is private to the logged in user
    let runtime_directory =
        env::var_os("XDG_RUNTIME_DIR").expect("failed to find socket: XDG_RUNTIME_DIR is not set");

    PathBuf::from(runtime_directory).join("amane.sock")
}
