mod call;
mod cargo;
mod clean;
mod compile;
mod dev;
mod library;
mod paths;
mod project;
mod run;
mod startup;
mod watch;

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();

    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();

    match words.as_slice() {
        ["startup"] => startup::run(false),

        ["startup", "--example"] => startup::run(true),

        ["compile"] => compile::run(),

        ["run"] => run::run(),

        ["dev"] => dev::run(),

        ["clean"] => clean::run(false),

        ["clean", "--yes"] => clean::run(true),

        ["ipc", "call", name, ..] => call::run(name, &arguments[3..]),

        _ => usage(),
    }
}

fn usage() -> ExitCode {
    eprintln!("usage:");
    eprintln!("  amane startup                        create ~/.config/amane/src/main.rs");
    eprintln!("  amane startup --example              create a full example bar instead");
    eprintln!("  amane dev                           rebuild and restart on every save");
    eprintln!("  amane compile                        build the optimised shell");
    eprintln!("  amane run                            start the compiled shell");
    eprintln!(
        "  amane clean                          remove the build cache, keep the compiled shell"
    );
    eprintln!("  amane clean --yes                    the same, without asking first");
    eprintln!("  amane ipc call <name> [arguments...] call a handler in the running shell");

    ExitCode::FAILURE
}
