use crate::token::{Token, TokenType};
use log::*;
use regex::*;
use std::any::Any;
// use std::convert::From;

lazy_static! {
  static ref SEMICOLON: Regex = Regex::new(";").unwrap();
  static ref DECIMAL: Regex = Regex::new("\\.\\w+").unwrap();
  static ref POST_DECIMAL: Regex = Regex::new("\\s+[0-9A-Za-z\"$,\\s]+").unwrap();
  static ref EQUALS: Regex = Regex::new("[a-zA-Z_0-9]+[\\s]+=").unwrap();
  static ref POST_EQUALS: Regex = Regex::new("=[^;\\n\\r]+").unwrap();
  static ref COLON: Regex = Regex::new("\\w*:").unwrap();
  static ref COMMA: Regex = Regex::new(",").unwrap();
  static ref WHITESPACE: Regex = Regex::new("\\s").unwrap();
}

pub struct Factory {
  machine: StateMachine,
}

impl Factory {
  pub fn new() -> Self {
    Factory {
      machine: StateMachine::new(),
    }
  }

  pub fn run(&mut self, text: &str, line: usize) -> Vec<Token> {
    self.machine.run(text, line);
    self.machine.tokens.clone()
  }
}

struct StateMachine {
  state: Box<dyn State>,
  tokens: Vec<Token>,
  line: usize,
}

impl StateMachine {
  fn run(&mut self, text: &str, line: usize) {
    self.line = line;
    let mut text = text.to_owned();
    if text.contains(';') {
      let semi_loc = text.find(';').unwrap_or(text.len());
      text.replace_range(semi_loc.., "");
    }
    let op = self.state.run(&text, line);
    match op {
      Some(s) => {
        self.state = s;
        self.run(&text, line);
      }
      None => {
        self.tokens = self.state.get_tokens();
        self.state = Box::new(Start);
      }
    }
  }
}

impl StateMachine {
  fn new() -> Self {
    StateMachine {
      state: Box::new(Start),
      tokens: vec![],
      line: 0,
    }
  }
}

#[derive(Clone)]
struct MatchData {
  text: String,
  start: usize,
}

impl MatchData {
  fn from_match(m: Match) -> Self {
    MatchData {
      text: m.as_str().to_owned(),
      start: m.start(),
    }
  }

  fn get_text(&self) -> String {
    self.text.to_owned()
  }

  fn get_column(&self) -> usize {
    self.start
  }
}

pub trait State: Any {
  fn run(&mut self, text: &str, line: usize) -> Option<Box<dyn State>>;
  fn get_tokens(&self) -> Vec<Token>;
}

struct Start;

impl State for Start {
  fn run(&mut self, text: &str, _: usize) -> Option<Box<dyn State>> {
    if text.len() < 1 {
      return None;
    }
    let m = DECIMAL.find(&text);
    if m.is_some() {
      return Some(Box::new(Directive {
        tokens: vec![],
        found: MatchData::from_match(m.unwrap()),
      }));
    }
    let m = EQUALS.find(&text);
    if m.is_some() {
      return Some(Box::new(Assignment {
        tokens: vec![],
        found: MatchData::from_match(m.unwrap()),
      }));
    }
    let m = COLON.find(&text);
    if m.is_some() {
      return Some(Box::new(Label {
        tokens: vec![],
        found: MatchData::from_match(m.unwrap()),
      }));
    }
    Some(Box::new(Mnemonic { tokens: vec![] }))
  }

  fn get_tokens(&self) -> Vec<Token> {
    vec![]
  }
}

struct Directive {
  tokens: Vec<Token>,
  found: MatchData,
}

impl State for Directive {
  fn run(&mut self, _: &str, line: usize) -> Option<Box<dyn State>> {
    self.tokens.push(Token::new(
      TokenType::Directive,
      self.found.get_text(),
      line,
      self.found.get_column(),
    ));
    Some(Box::new(DirectiveValue {
      tokens: self.tokens.to_owned(),
    }))
  }

  fn get_tokens(&self) -> Vec<Token> {
    self.tokens.to_vec()
  }
}

struct DirectiveValue {
  tokens: Vec<Token>,
}

impl DirectiveValue {
  fn get_all_post_operands(
    &mut self,
    text: &str,
    line: usize,
    start: usize,
  ) -> Option<Box<dyn State>> {
    let commas = COMMA.split(text);
    let mut place = start;
    for s in commas {
      // debug!("{}", s);
      let trimmed = s.trim().to_owned();
      self
        .tokens
        .push(Token::new(TokenType::DirectiveValue, trimmed, line, place));
      place += s.len();
    }
    None
  }
}

impl State for DirectiveValue {
  fn run(&mut self, text: &str, line: usize) -> Option<Box<dyn State>> {
    let m = POST_DECIMAL.find(text).unwrap();
    let m_text = m.as_str().trim_start();
    let m_text = m_text.replace("\"", "");
    match COMMA.is_match(text) {
      true => self.get_all_post_operands(&m_text, line, m.start()),
      false => {
        self.tokens.push(Token::new(
          TokenType::DirectiveValue,
          m_text.to_owned(),
          line,
          m.start(),
        ));
        None
      }
    }
  }

  fn get_tokens(&self) -> Vec<Token> {
    self.tokens.to_vec()
  }
}

struct Mnemonic {
  tokens: Vec<Token>,
}

impl State for Mnemonic {
  fn run(&mut self, text: &str, line: usize) -> Option<Box<dyn State>> {
    let text = text.trim();
    let split = WHITESPACE.split(text);
    let mut place = 0;
    for (i, s) in split.enumerate() {
      let token_type = match i {
        0 => TokenType::Mnemonic,
        _ => TokenType::Operand,
      };
      self
        .tokens
        .push(Token::new(token_type, s.to_owned(), line, place));
      place += s.len();
    }
    None
  }

  fn get_tokens(&self) -> Vec<Token> {
    self.tokens.to_vec()
  }
}

struct Label {
  tokens: Vec<Token>,
  found: MatchData,
}

impl State for Label {
  fn run(&mut self, _: &str, line: usize) -> Option<Box<dyn State>> {
    self.tokens.push(Token::new(
      TokenType::Label,
      self.found.get_text(),
      line,
      self.found.get_column(),
    ));
    None
  }

  fn get_tokens(&self) -> Vec<Token> {
    self.tokens.to_vec()
  }
}

struct Assignment {
  tokens: Vec<Token>,
  found: MatchData,
}

impl State for Assignment {
  fn run(&mut self, _: &str, line: usize) -> Option<Box<dyn State>> {
    let text = self.found.get_text();
    let text = text.replace("=", "");
    let text = text.trim_end().to_owned();
    self.tokens.push(Token::new(
      TokenType::Assignment,
      text,
      line,
      self.found.get_column(),
    ));
    Some(Box::new(Value {
      tokens: self.tokens.to_owned(),
    }))
  }

  fn get_tokens(&self) -> Vec<Token> {
    self.tokens.to_vec()
  }
}

struct Value {
  tokens: Vec<Token>,
}

impl State for Value {
  fn run(&mut self, text: &str, line: usize) -> Option<Box<dyn State>> {
    let m = POST_EQUALS.find(text).unwrap();
    let text = m.as_str().replace("=", "");
    let text = text.trim_start();
    self.tokens.push(Token::new(
      TokenType::Value,
      text.to_owned(),
      line,
      m.start(),
    ));
    None
  }

  fn get_tokens(&self) -> Vec<Token> {
    self.tokens.to_vec()
  }
}
