use serde::Serialize;

use nmk::arch::detect_current_architecture;
use nmk::human_time::{seconds_since_build, HumanTime};

#[derive(Serialize)]
struct Info {
    nmk: Nmk,
    rustup: Rustup,
    toolchain: Toolchain,
}

#[derive(Serialize)]
struct Toolchain {
    rustc: &'static str,
    target: &'static str,
}

#[derive(Serialize)]
struct Rustup {
    get_architecture: String,
}

#[derive(Serialize)]
struct Nmk {
    version: Option<String>,
    build_on: Option<String>,
}

pub fn print_info() -> nmk::Result<()> {
    let version = get_version();
    let build_on = seconds_since_build().map(|secs| format!("{} ago", HumanTime::new(secs)));
    let info = Info {
        nmk: Nmk { version, build_on },
        rustup: Rustup {
            get_architecture: detect_current_architecture()?,
        },
        toolchain: Toolchain {
            rustc: env!("BUILD_RUSTC_VERSION"),
            target: env!("BUILD_TARGET"),
        },
    };
    println!("{}", toml::to_string_pretty(&info)?);
    Ok(())
}

fn get_version() -> Option<String> {
    if let Some(hash) = option_env!("GIT_SHORT_SHA") {
        Some(format!("#{}", hash))
    } else if cfg!(debug_assertions) {
        Some("development".to_owned())
    } else {
        None
    }
}
