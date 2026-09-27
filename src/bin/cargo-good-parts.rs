//! Entrypoint for `cargo good-parts`. Cargo invokes this binary as
//! `cargo-good-parts good-parts [ARGS]`, so the subcommand name is dropped.

fn main() -> std::process::ExitCode {
    let mut args: Vec<_> = std::env::args_os().collect();
    if args.get(1).is_some_and(|arg| arg == "good-parts") {
        args.remove(1);
    }
    good_parts::main_with(args)
}
