mod parser;

use emu_attack::*;
use flexi_logger::{default_format, Logger};
use log::debug;

fn main() {
  Logger::with_env_or_str("trace")
    .log_to_file()
    .directory("log_files")
    .duplicate_to_stdout(flexi_logger::Duplicate::All)
    .format(default_format)
    .start()
    .unwrap();
  let pattern = std::env::args().nth(1).expect("no pattern given");
  if &pattern == "parser" {
    debug!("Initialized in parser mode");
    let mut parser = parser::Parser::new();
  // parser.run();
  } else {
    debug!("Initialized in program mode");
    let program = vec![0xA9, 0x10, 0x69, 0x10];
    let mut cpu = CPU::new();
    cpu.run(program);
    debug!("{}", cpu);
  }
}
