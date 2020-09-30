use regex::*;
use std::convert::From;

#[derive(Debug)]
struct Token {
  typ: TokenType,
  value: String,
  line: usize,
  column: usize,
}

impl Token {
  fn new(typ: TokenType, val: String, line: usize, column: usize) -> Token {
    Token {
      typ: typ,
      value: val,
      line: line,
      column: column,
    }
  }
}

#[derive(Debug)]
enum TokenType {
  String,
  Integer,
}

struct Lexer;

impl Lexer {
  pub fn nextToken() {
    // stub
  }
}

struct Factory {
  machine: StateMachineWrapper,
}

impl Factory {
  fn new() -> Self {
    Factory {
      machine: StateMachineWrapper::Mnemonic(StateMachine::new()),
    }
  }
}

enum StateMachineWrapper {
  Mnemonic(StateMachine<Mnemonic>),
  AddressOp(StateMachine<AddressOp>),
  Newline(StateMachine<Newline>),
  Operand(StateMachine<Operand>),
}

impl StateMachineWrapper {
  fn step(mut self) -> Self {
    self = match self {
      StateMachineWrapper::Mnemonic(val) => StateMachineWrapper::AddressOp(val.into()),
      StateMachineWrapper::Operand(val) => StateMachineWrapper::Newline(val.into()),
      StateMachineWrapper::AddressOp(val) => StateMachineWrapper::Operand(val.into()),
      StateMachineWrapper::Newline(val) => StateMachineWrapper::Mnemonic(val.into()),
    };
    self
  }
}

struct StateMachine<S> {
  state: S,
}

impl StateMachine<Mnemonic> {
  fn new() -> Self {
    StateMachine { state: Mnemonic {} }
  }
}

impl From<StateMachine<Mnemonic>> for StateMachine<AddressOp> {
  fn from(_: StateMachine<Mnemonic>) -> StateMachine<AddressOp> {
    StateMachine {
      state: AddressOp {},
    }
  }
}

impl From<StateMachine<AddressOp>> for StateMachine<Operand> {
  fn from(_: StateMachine<AddressOp>) -> StateMachine<Operand> {
    StateMachine { state: Operand {} }
  }
}

impl From<StateMachine<Operand>> for StateMachine<Newline> {
  fn from(_: StateMachine<Operand>) -> StateMachine<Newline> {
    StateMachine { state: Newline {} }
  }
}

impl From<StateMachine<Newline>> for StateMachine<Mnemonic> {
  fn from(_: StateMachine<Newline>) -> StateMachine<Mnemonic> {
    StateMachine { state: Mnemonic {} }
  }
}

struct Mnemonic;
struct AddressOp;
struct Operand;
struct Newline;

/*
   state diagram
   START(multiple destinations) | MNEMONIC | OPERAND | LABEL | SEMICOLON | COMMENT | NEWLINE(end)
*/

fn main() {
  let mnemonic = Regex::new("[a-zA-Z]{3}").unwrap();
  let digits = Regex::new("[0-9]+").unwrap();
  let dollar = Regex::new("\\$").unwrap();
  let paren = Regex::new("\\(\\$").unwrap();
  let hash = Regex::new("\\#\\$").unwrap();
  let regexs = vec![hash, paren, dollar];
  let test_string = "ADC ($44,X)\n";
  let mut factory = Factory::new();
  let mut tokens: Vec<Token> = Vec::with_capacity(test_string.len());
  let counter = 0;
  loop {
    match factory.machine {
      StateMachineWrapper::Mnemonic(_) => {
        let found = mnemonic.find(test_string).unwrap();
        add_token(found, &mut tokens, counter, TokenType::String);
      }
      StateMachineWrapper::AddressOp(_) => {
        for reg in regexs.iter() {
          let is_found = reg.find(test_string);
          if let Some(val) = is_found {
            add_token(val, &mut tokens, counter, TokenType::String);
            break;
          }
        }
      }
      StateMachineWrapper::Operand(_) => {
        let found = digits.find(test_string).unwrap();
        add_token(found, &mut tokens, counter, TokenType::Integer);
      }
      StateMachineWrapper::Newline(_) => break,
    };
    factory.machine = factory.machine.step();
  }
  println!("{:?}", tokens);
}

fn add_token(m: regex::Match, v: &mut Vec<Token>, ln_nm: usize, token_type: TokenType) {
  println!("{}", m.as_str());
  let token = Token::new(token_type, m.as_str().to_owned(), ln_nm, m.start());
  v.push(token);
}
