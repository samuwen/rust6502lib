use std::fmt::*;

#[derive(Clone)]
pub struct Token {
  typ: TokenType,
  value: String,
  line: usize,
  column: usize,
}

impl std::fmt::Debug for Token {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    f.debug_struct("Token")
      .field("value", &self.value)
      .field("type", &self.typ)
      .finish()
  }
}

impl Token {
  pub fn new(typ: TokenType, val: String, line: usize, column: usize) -> Token {
    Token {
      typ: typ,
      value: val,
      line: line,
      column: column,
    }
  }

  pub fn get_value(&self) -> &str {
    &self.value
  }
}

impl Ord for Token {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.line.cmp(&other.line)
  }
}

impl PartialOrd for Token {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Eq for Token {}

impl PartialEq for Token {
  fn eq(&self, other: &Self) -> bool {
    self.typ == other.typ && self.value == other.value && self.line == other.line
  }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenType {
  Assignment,
  Directive,
  DirectiveValue,
  Label,
  Mnemonic,
  Operand,
  Value,
}
