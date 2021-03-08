mod cmdline;
mod commands;
mod entrypoint;
mod logging;
mod path_vec;
mod terminal;
mod tmux;
mod utils;
mod version;
mod zsh;

fn main() -> nmk::Result<()> {
    let cmd_opt = cmdline::parse();
    logging::setup(cmd_opt.verbosity);
    log::debug!("Command line options: {:#?}", cmd_opt);
    if let Some(cmd) = cmd_opt.cmd {
        use cmdline::SubCommand::*;
        match cmd {
            Backup => commands::backup::backup()?,
            Info => commands::info::print_info()?,
            Completions(ref c) => commands::completion::gen_completion(c),
        }
    } else {
        entrypoint::main(cmd_opt)?;
    }
    Ok(())
}
