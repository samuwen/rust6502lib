#[derive(Debug)]
pub struct Token {
  typ: TokenType,
  value: String,
  line: usize,
  column: usize,
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

  pub fn get(&self) -> &str {
    &self.value
  }
}

#[derive(Debug)]
pub enum TokenType {
  String,
  AddressType,
  Operand,
}
