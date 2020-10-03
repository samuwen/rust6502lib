use regex::*;
use std::convert::From;

lazy_static! {
  static ref SEMICOLON: Regex = Regex::new("\\s;").unwrap();
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

pub struct Factory {
  machine: StateMachineWrapper,
}

impl Factory {
  pub fn new() -> Self {
    Factory {
      machine: StateMachineWrapper::Mnemonic(StateMachine::new()),
    }
  }

  pub fn get_machine(&self) -> &StateMachineWrapper {
    &self.machine
  }
}

enum StateMachineWrapper {
  Start(StateMachine<Start>),
  Directive(StateMachine<Directive>),
  Comment(StateMachine<Comment>),
  Text(StateMachine<Text>),
  Assignment(StateMachine<Assignment>),
  Value(StateMachine<Value>),
  Label(StateMachine<Label>),
  Mnemonic(StateMachine<Mnemonic>),
  Operator(StateMachine<Operator>),
  Operand(StateMachine<Operand>),
  Newline(StateMachine<Newline>),
}

impl StateMachineWrapper {
  fn step(mut self) -> Self {
    self = match self {
      StateMachineWrapper::Start(val) => StateMachineWrapper::Directive(val.into()),
      StateMachineWrapper::Directive(val) => StateMachineWrapper::Directive(val.into()),
      StateMachineWrapper::Comment(val) => StateMachineWrapper::Comment(val.into()),
      StateMachineWrapper::Text(val) => StateMachineWrapper::Text(val.into()),
      StateMachineWrapper::Assignment(val) => StateMachineWrapper::Assignment(val.into()),
      StateMachineWrapper::Value(val) => StateMachineWrapper::Value(val.into()),
      StateMachineWrapper::Label(val) => StateMachineWrapper::Label(val.into()),
      StateMachineWrapper::Mnemonic(val) => StateMachineWrapper::Operator(val.into()),
      StateMachineWrapper::Operator(val) => StateMachineWrapper::Operand(val.into()),
      StateMachineWrapper::Operand(val) => StateMachineWrapper::Newline(val.into()),
      StateMachineWrapper::Newline(val) => StateMachineWrapper::Newline(val),
    };
    self
  }

  fn reset(mut self) -> Self {
    self = StateMachineWrapper::Mnemonic(StateMachine::new());
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

impl From<StateMachine<Start>> for StateMachine<Mnemonic> {
  fn from(_: StateMachine<Start>) -> StateMachine<Mnemonic> {
    StateMachine { state: Mnemonic {} }
  }
}

impl From<StateMachine<Start>> for StateMachine<Comment> {
  fn from(_: StateMachine<Start>) -> StateMachine<Comment> {
    StateMachine { state: Comment {} }
  }
}

impl From<StateMachine<Start>> for StateMachine<Directive> {
  fn from(_: StateMachine<Start>) -> StateMachine<Directive> {
    StateMachine {
      state: Directive {},
    }
  }
}

impl From<StateMachine<Start>> for StateMachine<Assignment> {
  fn from(_: StateMachine<Start>) -> StateMachine<Assignment> {
    StateMachine {
      state: Assignment {},
    }
  }
}

impl From<StateMachine<Start>> for StateMachine<Label> {
  fn from(_: StateMachine<Start>) -> StateMachine<Label> {
    StateMachine { state: Label {} }
  }
}

impl From<StateMachine<Label>> for StateMachine<Newline> {
  fn from(_: StateMachine<Label>) -> StateMachine<Newline> {
    StateMachine { state: Newline {} }
  }
}

impl From<StateMachine<Directive>> for StateMachine<Newline> {
  fn from(_: StateMachine<Directive>) -> StateMachine<Newline> {
    StateMachine { state: Newline {} }
  }
}

impl From<StateMachine<Comment>> for StateMachine<Text> {
  fn from(_: StateMachine<Comment>) -> StateMachine<Text> {
    StateMachine { state: Text {} }
  }
}

impl From<StateMachine<Text>> for StateMachine<Newline> {
  fn from(_: StateMachine<Text>) -> StateMachine<Newline> {
    StateMachine { state: Newline {} }
  }
}

impl From<StateMachine<Mnemonic>> for StateMachine<Operator> {
  fn from(_: StateMachine<Mnemonic>) -> StateMachine<Operator> {
    StateMachine { state: Operator {} }
  }
}

impl From<StateMachine<Operator>> for StateMachine<Operand> {
  fn from(_: StateMachine<Operator>) -> StateMachine<Operand> {
    StateMachine { state: Operand {} }
  }
}

impl From<StateMachine<Operand>> for StateMachine<Newline> {
  fn from(_: StateMachine<Operand>) -> StateMachine<Newline> {
    StateMachine { state: Newline {} }
  }
}

impl From<StateMachine<Assignment>> for StateMachine<Value> {
  fn from(_: StateMachine<Assignment>) -> StateMachine<Value> {
    StateMachine { state: Value {} }
  }
}

impl From<StateMachine<Value>> for StateMachine<Newline> {
  fn from(_: StateMachine<Value>) -> StateMachine<Newline> {
    StateMachine { state: Newline {} }
  }
}

trait State {
  fn run(&self, text: &str) -> Box<dyn State>;
}

struct Start;

impl Start {
  fn check_match(found: Option<Match>) -> bool {
    match found {
      Some(v) => v.start() == 0,
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
  fn run(&self, text: &str) -> Box<dyn State> {
    let text = text.trim_start();
    if Start::starts_with_semi(text) {
      return Box::new(Comment);
    }
    if Start::starts_with_decimal(text) {
      return Box::new(Directive);
    }
    if Start::starts_with_equals_expression(text) {
      return Box::new(Assignment);
    }
    if Start::starts_with_colon_expression(text) {
      return Box::new(Label);
    }
    // else
    Box::new(Mnemonic)
  }
}

struct Comment;

impl State for Comment {
  fn run(&self, text: &str) -> Box<dyn State> {
    Box::new(Start {})
  }
}

struct Directive;

impl State for Directive {
  fn run(&self, text: &str) -> Box<dyn State> {
    Box::new(Start {})
  }
}

struct Mnemonic;

impl State for Mnemonic {
  fn run(&self, text: &str) -> Box<dyn State> {
    Box::new(Start {})
  }
}

struct Label;

impl State for Label {
  fn run(&self, text: &str) -> Box<dyn State> {
    Box::new(Start {})
  }
}

struct Assignment;

impl State for Assignment {
  fn run(&self, text: &str) -> Box<dyn State> {
    Box::new(Start {})
  }
}

struct Text;
struct Value;
struct Operator;
struct Operand;
struct Newline;

/*

  loop {
    match factory.get_machine() {
      StateMachineWrapper::Mnemonic(_) => {
        let found = mnemonic.find(&test_string).unwrap();
        add_token(found, &mut tokens, counter, TokenType::String);
      }
      StateMachineWrapper::AddressOp(_) => {
        for reg in regexs.iter() {
          let is_found = reg.find(&test_string);
          if let Some(val) = is_found {
            add_token(val, &mut tokens, counter, TokenType::AddressType);
            break;
          }
        }
      }
      StateMachineWrapper::Operand(_) => {
        let found = digits.find(&test_string).unwrap();
        add_token(found, &mut tokens, counter, TokenType::Operand);
      }
      StateMachineWrapper::Newline(_) => {
        opcodes.push(get_opcode_from_map(&tokens[0], &tokens[1]));
        let operand = u8::from_str_radix(tokens[3].get(), 16).unwrap();
        opcodes.push(operand);
        factory.machine = factory.machine.reset();
      }
    };
    factory.machine = factory.machine.step();
  }
*/
