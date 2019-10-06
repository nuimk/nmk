use std::env;
use std::fs::File;
use std::path::PathBuf;

use simplelog::{CombinedLogger, TermLogger, LevelFilter, TerminalMode};
use crate::core::set_env;
use crate::pathenv::PathVec;

pub fn setup_logging(debug: bool) {
    let log_level = if debug { LevelFilter::Debug } else { LevelFilter::Info };
    CombinedLogger::init(
        vec![
            TermLogger::new(log_level, simplelog::Config::default(), TerminalMode::Mixed).unwrap()
        ]
    ).unwrap();
}

pub fn setup_environment(nmk_dir: &PathBuf) {
    let init_vim = nmk_dir.join("vim").join("init.vim");
    let zdotdir = nmk_dir.join("zsh");
    set_env("NMK_DIR", nmk_dir);

    if let Some(path) = init_vim
        .to_str()
        .map(|s|
            s.to_string().replace(" ", r"\ ")
        ) {
        set_env("VIMINIT", format!("source {}", path));
    }
    set_env("ZDOTDIR", zdotdir);

    env::remove_var("VIRTUAL_ENV");

    set_env("NMK_BIN", env::current_exe().unwrap());
}

pub fn setup_preferred_editor() {
    const EDITOR: &str = "EDITOR";
    if env::var_os(EDITOR).is_none() {
        let mut editors = ["nvim", "vim"].iter();
        if let Some(editor) = editors.find(|bin| which::which(bin).is_ok()) {
            set_env(EDITOR, editor);
            debug!("using {} as prefer editor", editor);
        }
    }
}

pub fn setup_path(nmk_dir: &PathBuf) {
    const PATH: &str = "PATH";
    let mut p = PathVec::parse(env::var_os(PATH).expect("$PATH not found"));
    p.push_front(nmk_dir.join("local").join("bin"));
    p.push_front(nmk_dir.join("bin"));
    p = p.unique().no_version_managers();
    if log_enabled!(log::Level::Debug) {
        for (i, path) in p.iter().enumerate() {
            debug!("{}[{}]={:?}", PATH, i + 1, path);
        }
    }
    set_env(PATH, p.make());
}

pub fn setup_ld_library_path(nmk_dir: &PathBuf) {
    const LD: &str = "LD_LIBRARY_PATH";

    let local_lib_dir = nmk_dir.join("local").join("lib");
    if local_lib_dir.exists() {
        let mut ps = match env::var_os(LD) {
            Some(path) => {
                debug!("{}: {:?}", LD, path);
                PathVec::parse(path)
            }
            None => PathVec::new(),
        };
        ps.push_front(local_lib_dir);
        let next_ld = ps.make();
        set_env(LD, next_ld);
    }
}

pub fn dir() -> PathBuf {
    const NMK_DIR: &str = "NMK_DIR";
    let path = match env::var_os(NMK_DIR) {
        Some(s) => PathBuf::from(s),
        None => dirs::home_dir()
            .expect("Can't find home directory")
            .join(".nmk"),
    };
    if !path.exists() {
        panic!(format!("{:?} doesn't exist", path));
    }
    path
}

pub fn print_message_of_the_day() {
    let mut stdout = std::io::stdout();
    ["/var/run/motd.dynamic", "/etc/motd"]
        .iter()
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .flat_map(File::open)
        .for_each(|mut f| {
            std::io::copy(&mut f, &mut stdout).expect("fail to print motd");
        });
}