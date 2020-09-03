use emu_attack::*;
use flexi_logger::{opt_format, Logger};

fn main() {
    Logger::with_env_or_str("info")
        .log_to_file()
        .directory("log_files")
        .format(opt_format)
        .start()
        .unwrap();
    let mut cpu = CPU::new();
    cpu.reset();
}
