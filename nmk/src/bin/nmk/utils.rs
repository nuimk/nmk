use crate::cmdline::CmdOpt;

pub fn print_usage_time(cmd_opt: &CmdOpt) {
    let elapsed = cmd_opt.start_time.elapsed().as_millis();
    if cmd_opt.usage {
        println!("{} ms.", elapsed);
    } else {
        log::debug!("Usage time: {} ms.", elapsed);
    }
}
