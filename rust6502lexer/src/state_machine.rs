use crate::token::{Token, TokenType};
use log::*;
use regex::*;
use std::any::Any;
use std::convert::From;

lazy_static! {
  static ref SEMICOLON: Regex = Regex::new(";").unwrap();
  static ref DECIMAL: Regex = Regex::new("\\.\\w").unwrap();
  static ref EQUALS: Regex = Regex::new("[^;]+=").unwrap();
  static ref COLON: Regex = Regex::new("\\w*:").unwrap();
  static ref MNEMONIC: Regex = Regex::new("[a-zA-Z]{3}").unwrap();
  static ref DIGITS: Regex = Regex::new("[0-9]+").unwrap();
  static ref DOLLAR: Regex = Regex::new("\\$").unwrap();
  static ref PAREN: Regex = Regex::new("\\(\\$").unwrap();
  static ref HASH: Regex = Regex::new("\\#\\$").unwrap();
}

/// Checks if code needs to be commented out
///
/// Returns true if a semicolon is found before other operands
fn is_semi_interrupting(text: &str, test: Match) -> bool {
  match SEMICOLON.find(text) {
    Some(loc) => test.start() < loc.start(),
    None => false,
  }
}

fn logz(text: &str, message: &str) {
  // debug!("{}: {}", message, text);
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
    let op = self.state.run(text, line);
    match op {
      Some(s) => {
        self.state = s;
        self.run(text, line);
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

pub trait State: Any {
  fn run(&mut self, text: &str, line: usize) -> Option<Box<dyn State>>;
  fn get_tokens(&self) -> Vec<Token>;
}

struct Start;

impl Start {
  fn check_match(found: Option<Match>) -> bool {
    match found {
      Some(v) => {
        return v.start() == 0;
      }
      None => false,
    }
  }
  /// Checks if the first character is a semicolon indicating the whole line is a comment
  fn starts_with_semi(text: &str) -> bool {
    Start::check_match(SEMICOLON.find(text))
  }

  fn starts_with_decimal(text: &str) -> bool {
    Start::check_match(DECIMAL.find(text))
  }

  fn starts_with_equals_expression(text: &str) -> bool {
    Start::check_match(EQUALS.find(text))
  }

  fn starts_with_colon_expression(text: &str) -> bool {
    Start::check_match(COLON.find(text))
  }
}

impl State for Start {
  fn run(&mut self, text: &str, _: usize) -> Option<Box<dyn State>> {
    let text = text.trim_start();
    if Start::starts_with_semi(text) {
      logz(text, "SEMI");
      return None;
    }
    if Start::starts_with_decimal(text) {
      logz(text, "DIRECTIVE");
      return Some(Box::new(Directive { tokens: vec![] }));
    }
    if Start::starts_with_equals_expression(text) {
      logz(text, "ASSIGNMENT");
      return Some(Box::new(Assignment { tokens: vec![] }));
    }
    if Start::starts_with_colon_expression(text) {
      return Some(Box::new(Label { tokens: vec![] }));
    }
    Some(Box::new(Mnemonic { tokens: vec![] }))
  }

  fn get_tokens(&self) -> Vec<Token> {
    vec![]
  }
}

struct Directive {
  tokens: Vec<Token>,
}

impl State for Directive {
  fn run(&mut self, text: &str, line: usize) -> Option<Box<dyn State>> {
    None
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
    None
  }

  fn get_tokens(&self) -> Vec<Token> {
    self.tokens.to_vec()
  }
}

struct Label {
  tokens: Vec<Token>,
}

impl State for Label {
  fn run(&mut self, text: &str, line: usize) -> Option<Box<dyn State>> {
    let val = COLON.find(text).unwrap();
    self.tokens.push(Token::new(
      TokenType::Directive,
      val.as_str().to_owned(),
      line,
      val.start(),
    ));
    None
  }

  fn get_tokens(&self) -> Vec<Token> {
    self.tokens.to_vec()
  }
}

struct Assignment {
  tokens: Vec<Token>,
}

impl State for Assignment {
  fn run(&mut self, text: &str, line: usize) -> Option<Box<dyn State>> {
    None
  }

  fn get_tokens(&self) -> Vec<Token> {
    self.tokens.to_vec()
  }
}

struct Text;
struct Value;
struct Operator;
struct Operand;
struct Newline;

// impl From<StateMachine<Start>> for StateMachine<Mnemonic> {
//   fn from(_: StateMachine<Start>) -> StateMachine<Mnemonic> {
//     StateMachine { state: Mnemonic {} }
//   }
// }

// impl From<StateMachine<Start>> for StateMachine<Comment> {
//   fn from(_: StateMachine<Start>) -> StateMachine<Comment> {
//     StateMachine { state: Comment {} }
//   }
// }

// impl From<StateMachine<Start>> for StateMachine<Directive> {
//   fn from(_: StateMachine<Start>) -> StateMachine<Directive> {
//     StateMachine {
//       state: Directive {},
//     }
//   }
// }

// impl From<StateMachine<Start>> for StateMachine<Assignment> {
//   fn from(_: StateMachine<Start>) -> StateMachine<Assignment> {
//     StateMachine {
//       state: Assignment {},
//     }
//   }
// }

// impl From<StateMachine<Start>> for StateMachine<Label> {
//   fn from(_: StateMachine<Start>) -> StateMachine<Label> {
//     StateMachine { state: Label {} }
//   }
// }

// // impl From<StateMachine<Label>> for StateMachine<Newline> {
// //   fn from(_: StateMachine<Label>) -> StateMachine<Newline> {
// //     StateMachine { state: Newline {} }
// //   }
// // }

// // impl From<StateMachine<Directive>> for StateMachine<Newline> {
// //   fn from(_: StateMachine<Directive>) -> StateMachine<Newline> {
// //     StateMachine { state: Newline {} }
// //   }
// // }

// // impl From<StateMachine<Comment>> for StateMachine<Text> {
// //   fn from(_: StateMachine<Comment>) -> StateMachine<Text> {
// //     StateMachine { state: Text {} }
// //   }
// // }

// // impl From<StateMachine<Text>> for StateMachine<Newline> {
// //   fn from(_: StateMachine<Text>) -> StateMachine<Newline> {
// //     StateMachine { state: Newline {} }
// //   }
// // }

// // impl From<StateMachine<Mnemonic>> for StateMachine<Operator> {
// //   fn from(_: StateMachine<Mnemonic>) -> StateMachine<Operator> {
// //     StateMachine { state: Operator {} }
// //   }
// // }

// // impl From<StateMachine<Operator>> for StateMachine<Operand> {
// //   fn from(_: StateMachine<Operator>) -> StateMachine<Operand> {
// //     StateMachine { state: Operand {} }
// //   }
// // }

// // impl From<StateMachine<Operand>> for StateMachine<Newline> {
// //   fn from(_: StateMachine<Operand>) -> StateMachine<Newline> {
// //     StateMachine { state: Newline {} }
// //   }
// // }

// // impl From<StateMachine<Assignment>> for StateMachine<Value> {
// //   fn from(_: StateMachine<Assignment>) -> StateMachine<Value> {
// //     StateMachine { state: Value {} }
// //   }
// // }

// // impl From<StateMachine<Value>> for StateMachine<Newline> {
// //   fn from(_: StateMachine<Value>) -> StateMachine<Newline> {
// //     StateMachine { state: Newline {} }
// //   }
// // }
