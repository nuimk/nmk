use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};

use log::LevelFilter;
use simplelog::{SimpleLogger, TerminalMode, TermLogger};
use structopt::StructOpt;

use crate::zsh;
use crate::cmdline::Opt;
use crate::common::env_var::{EDITOR, LD_LIBRARY_PATH, NMK_DIR, PATH};
use crate::core::set_env;
use crate::pathenv::PathVec;
use crate::tmux::Tmux;

pub fn setup_logging(debug: bool) {
    let log_level = if debug { LevelFilter::Debug } else { LevelFilter::Info };
    let config = simplelog::ConfigBuilder::new()
        .set_thread_level(LevelFilter::Trace)
        .set_target_level(LevelFilter::Trace)
        .build();
    if TermLogger::init(log_level,
                        config.clone(),
                        TerminalMode::Stderr).is_err() {
        SimpleLogger::init(log_level, config).expect("Unable to setup logging");
    }
}

pub fn setup_environment(nmk_dir: &PathBuf) {
    let init_vim = nmk_dir.join("vim").join("init.vim");
    let zdotdir = nmk_dir.join("zsh");
    set_env(NMK_DIR, nmk_dir);

    let quote_first_space = |s: &str| s.to_string().replace(" ", r"\ ");
    if let Some(path) = init_vim.to_str().map(quote_first_space) {
        set_env("VIMINIT", format!("source {}", path));
    }
    set_env("ZDOTDIR", zdotdir);

    env::remove_var("VIRTUAL_ENV");

    set_env("NMK_BIN", env::current_exe().expect("fail to get full path to executable"));
}

pub fn setup_preferred_editor() {
    const PREFERRED_EDITORS: &[&str] = &["nvim", "vim"];
    let mut editors: Vec<&OsStr> = Vec::with_capacity(PREFERRED_EDITORS.len() + 1);
    editors.extend(PREFERRED_EDITORS.iter().map(|e| -> &OsStr { e.as_ref() }));

    let env_editor;
    if let Some(editor) = env::var_os(EDITOR) {
        env_editor = editor;
        editors.insert(0, &env_editor);
    }
    if let Some(editor) = editors.into_iter().find(|bin| which::which(bin).is_ok()) {
        set_env(EDITOR, editor);
        log::debug!("using {:?} as preferred editor", editor);
    } else {
        env::remove_var(EDITOR);
    }
}

pub fn setup_path(nmk_dir: &PathBuf) {
    let mut bin_path = PathVec::parse(env::var_os(PATH).expect("$PATH not found"));
    bin_path.push_front(nmk_dir.join("local").join("bin"));
    bin_path.push_front(nmk_dir.join("bin"));
    bin_path = bin_path.unique().no_version_managers();
    log::debug!("{} = {:#?}", PATH, bin_path);
    set_env(PATH, bin_path.make());
}

pub fn setup_ld_library_path(nmk_dir: &PathBuf) {
    let local_lib_dir = nmk_dir.join("local").join("lib");
    if local_lib_dir.exists() {
        let mut lib_path = match env::var_os(LD_LIBRARY_PATH) {
            Some(value) => PathVec::parse(value),
            None => PathVec::new(),
        };
        lib_path.push_front(local_lib_dir);
        log::debug!("{} = {:#?}", LD_LIBRARY_PATH, lib_path);
        let next_ld = lib_path.make();
        set_env(LD_LIBRARY_PATH, next_ld);
    }
}

pub fn nmk_dir() -> PathBuf {
    let path = match env::var_os(NMK_DIR) {
        Some(s) => PathBuf::from(s),
        None => dirs::home_dir()
            .expect("Can't find home directory")
            .join(".nmk"),
    };
    assert!(path.exists(), "{:?} doesn't exist", path);
    path
}

pub fn display_message_of_the_day() {
    let mut stdout = std::io::stdout();
    ["/var/run/motd.dynamic", "/etc/motd"]
        .iter()
        .map(Path::new)
        .filter(|p| p.exists())
        .flat_map(File::open)
        .for_each(|mut f| {
            std::io::copy(&mut f, &mut stdout).expect("fail to print motd");
        });
}

pub fn is_dev_machine() -> bool {
    env::var_os("DISPLAY").is_some() &&
        env::var_os("WINDOWID").is_some()
}

pub fn main() {
    let start = std::time::Instant::now();
    let arg: Opt = Opt::from_args();
    setup_logging(arg.debug);

    if arg.ssh {
        display_message_of_the_day();
    }

    let nmk_dir = nmk_dir();
    log::debug!("nmk_dir={:?}", nmk_dir);
    setup_ld_library_path(&nmk_dir);
    setup_path(&nmk_dir);

    let tmux = Tmux::new(&nmk_dir);
    log::debug!("tmux bin = {:?}", tmux.bin);
    log::debug!("tmux version = {}", tmux.version);

    setup_environment(&nmk_dir);
    setup_preferred_editor();
    zsh::setup(&arg, &nmk_dir);
    if arg.login {
        tmux.login_shell(&arg, &start);
    } else {
        tmux.setup_environment(&arg);
        tmux.exec(&arg, &start);
    }
}