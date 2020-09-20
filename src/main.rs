use emu_attack::*;
use flexi_logger::{detailed_format, Logger};
use log::debug;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
  Logger::with_env_or_str("trace")
    .log_to_file()
    .directory("log_files")
    .duplicate_to_stdout(flexi_logger::Duplicate::All)
    .format(detailed_format)
    .start()
    .unwrap();
  let pattern = std::env::args().nth(1).expect("no pattern given");
  if &pattern == "parser" {
    debug!("Initialized in parser mode");
    debug!("Disabled for the moment");
  // let mut parser = parser::Parser::new();
  // parser.run();
  } else {
    let (tx, rx) = mpsc::channel();
    let clock = thread::spawn(move || loop {
      thread::sleep(Duration::from_micros(1790));
      tx.send(true).unwrap();
    });
    debug!("Initialized in program mode");
    let program = vec![0xA9, 0x10, 0x69, 0x10];
    let mut cpu = CPU::new(rx);
    let running = cpu.run(program, None);
    if !running {
      clock.join().unwrap();
    }
  }
}
