mod lexer;
mod utils;

use flexi_logger::*;
use lexer::{Lexer, TokenType};
use log::*;
use std::fs::read_to_string;

fn main() {
  Logger::with_env_or_str("trace")
    .duplicate_to_stdout(flexi_logger::Duplicate::All)
    .format(colored_default_format)
    .start()
    .unwrap();
  let file_string = read_to_string("examples/nesgame.a65").expect("Failed to find example file");
  let mut lexer = Lexer::new(Box::new(file_string.chars().peekable()));
  loop {
    let token = lexer.next_token();
    debug!("{:?}", token);
    if *token.get_type() == TokenType::EndOfInput {
      break;
    }
  }
}
