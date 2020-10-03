#[macro_use]
extern crate lazy_static;

mod state_machine;
mod token;

use regex::*;
use state_machine::Factory;
use std::fs::read_to_string;
use token::{Token, TokenType};

fn main() {
  // let regexs = vec![hash, paren, dollar];
  let file_string = read_to_string("examples/nesgame.a65").expect("Failed to find example file");
  let mut factory = Factory::new();
  let mut tokens: Vec<Token> = Vec::with_capacity(file_string.len());
  let mut opcodes: Vec<u8> = Vec::with_capacity(file_string.len());
  for chunk in file_string.split("\n") {
    println!("{}", chunk);
  }
}

fn add_token(m: regex::Match, v: &mut Vec<Token>, ln_nm: usize, token_type: TokenType) {
  let token = Token::new(token_type, m.as_str().to_owned(), ln_nm, m.start());
  v.push(token);
}

fn get_opcode_from_map(mne_token: &Token, address_token: &Token) -> u8 {
  match mne_token.get() {
    "ADC" => match address_token.get() {
      "#$" => 0x69,
      _ => 0,
    },
    _ => 0,
  }
}
