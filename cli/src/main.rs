mod call;
mod cargo;
mod clean;
mod compile;
mod dev;
mod library;
mod new;
mod paths;
mod project;
mod restart;
mod run;
mod target;
mod watch;

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();

    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();

    match words.as_slice() {
        ["new"] => new::run(None, false),

        ["new", "--example"] => new::run(None, true),

        ["new", folder] => new::run(Some(folder), false),

        ["new", folder, "--example"] => new::run(Some(folder), true),

        ["startup"] => startup(false),

        ["startup", "--example"] => startup(true),

        ["target"] => target::run(None),

        ["target", folder] => target::run(Some(folder)),

        ["compile"] => compile::run(false),

        ["compile", "--restart"] => compile::run(true),

        ["run"] => run::run(),

        ["restart"] => restart::run(),

        ["dev"] => dev::run(),

        ["clean"] => clean::run(false),

        ["clean", "--yes"] => clean::run(true),

        ["ipc", "call", name, ..] => call::run(name, &arguments[3..]),

        _ => usage(),
    }
}

// the old name for `amane new`, kept for a while so existing setups don't break
fn startup(example: bool) -> ExitCode {
    eprintln!(
        "warning: `amane startup` is deprecated and will be removed, use `amane new` instead"
    );

    new::run(None, example)
}

fn usage() -> ExitCode {
    eprintln!("usage:");
    eprintln!("  amane new [folder]                   create a shell, in ~/.config/amane by default");
    eprintln!("  amane new [folder] --example         create a full example bar instead");
    eprintln!("  amane target [folder]                show or set the folder your shell lives in");
    eprintln!("  amane dev                           rebuild and restart on every save");
    eprintln!("  amane compile                        build the optimised shell");
    eprintln!("  amane compile --restart              build it, then restart the running shell");
    eprintln!("  amane run                            start the compiled shell");
    eprintln!(
        "  amane restart                        stop the running shell and start the compiled one"
    );
    eprintln!(
        "  amane clean                          remove the build cache, keep the compiled shell"
    );
    eprintln!("  amane clean --yes                    the same, without asking first");
    eprintln!("  amane ipc call <name> [arguments...] call a handler in the running shell");

    ExitCode::FAILURE
}
