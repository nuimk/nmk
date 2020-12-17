use std::fs::File;
use std::io::Write;

use structopt::StructOpt;

use nmk::bin_name::NMK;

use crate::cmdline::{Completion, Opt};

pub fn gen_completion(completion: &Completion) {
    let mut write: Box<dyn Write> = match completion.output {
        Some(ref p) => Box::new(File::create(p).expect("Cannot create completion output file")),
        None => Box::new(std::io::stdout()),
    };
    Opt::clap().gen_completions_to(NMK, completion.shell, &mut write);
}
