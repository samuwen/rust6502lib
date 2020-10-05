#[derive(Debug)]
pub struct Token {
  token_type: TokenType,
  line: usize,
  value: String,
  column: usize,
}

impl Token {
  pub fn new(token_type: TokenType) -> TokenBuilder {
    TokenBuilder {
      token_type: token_type,
      line: 0,
      value: String::from(""),
      column: 0,
    }
  }

  pub fn get_type(&self) -> &TokenType {
    &self.token_type
  }

  pub fn get_value(&self) -> &String {
    &self.value
  }

  pub fn get_line(&self) -> &usize {
    &self.line
  }

  pub fn get_column(&self) -> &usize {
    &self.column
  }
}

pub struct TokenBuilder {
  token_type: TokenType,
  line: usize,
  value: String,
  column: usize,
}

impl TokenBuilder {
  pub fn build(self) -> Token {
    Token {
      token_type: self.token_type,
      line: self.line,
      value: self.value,
      column: self.column,
    }
  }

  pub fn line(mut self, line: usize) -> TokenBuilder {
    self.line = line;
    self
  }

  pub fn value(mut self, value: String) -> TokenBuilder {
    self.value = value;
    self
  }

  pub fn column(mut self, column: usize) -> TokenBuilder {
    self.column = column;
    self
  }
}

#[derive(Eq, PartialEq, Debug)]
pub enum TokenType {
  // Sequences
  Identifier,
  Label,
  Number,
  String,
  Command,

  // Operators
  Assignment,
  Hash,
  LoByte,
  HiByte,
  Comma,
  OParen,
  CParen,

  // End of input
  EndOfInput,
}

impl TokenType {
  pub fn from_ch(ch: char) -> TokenType {
    match ch {
      '=' => TokenType::Assignment,
      '#' => TokenType::Hash,
      '<' => TokenType::LoByte,
      '>' => TokenType::HiByte,
      '(' => TokenType::OParen,
      ')' => TokenType::CParen,
      _ => TokenType::Comma,
    }
  }
}
