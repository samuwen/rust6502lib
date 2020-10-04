#[macro_use]
extern crate lazy_static;

mod state_machine;
mod token;

use flexi_logger::*;
use log::*;
use state_machine::Factory;
use std::fs::read_to_string;
use token::Token;

fn main() {
  Logger::with_env_or_str("trace")
    .duplicate_to_stdout(flexi_logger::Duplicate::All)
    .format(colored_default_format)
    .start()
    .unwrap();
  // let regexs = vec![hash, paren, dollar];
  let file_string = read_to_string("examples/nesgame.a65").expect("Failed to find example file");
  let mut factory = Factory::new();
  let split = file_string.split("\n");
  let mut tokens: Vec<Token> = Vec::with_capacity(split.size_hint().0);
  for (i, chunk) in split.enumerate() {
    // Prune out newlines
    if chunk.len() > 1 {
      let mut result_tokens = factory.run(chunk, i);
      // if result_tokens.len() > 0 {
      //   info!("{:?}", result_tokens);
      // }
      tokens.append(&mut result_tokens);
    }
  }
}
