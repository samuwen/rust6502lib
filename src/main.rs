mod parser;

use flexi_logger::{default_format, Logger};

fn main() {
  Logger::with_env_or_str("trace")
    .log_to_file()
    .directory("log_files")
    .duplicate_to_stdout(flexi_logger::Duplicate::All)
    .format(default_format)
    .start()
    .unwrap();
  let mut parser = parser::Parser::new();
  parser.run();
}
