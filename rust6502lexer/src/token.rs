#[derive(Debug)]
/// A token representing some piece of text
///
/// Contains the text and some metadata about the text itself.
/// # Examples
/// ```
/// use rust6502lexer::{Token, TokenType};
/// let token = Token::new(TokenType::Identifier).build();
/// ```
pub struct Token {
  token_type: TokenType,
  line: usize,
  value: String,
  column: usize,
}

impl Token {
  /// Creates and returns a new token builder
  ///
  /// Builder used for convenience. Just invoke build to complete.
  /// # Examples
  /// ```
  /// use rust6502lexer::{Token, TokenType};
  /// let token_builder = Token::new(TokenType::Identifier);
  /// let token = token_builder.line(25).column(25).build();
  /// ```
  pub fn new(token_type: TokenType) -> TokenBuilder {
    TokenBuilder {
      token_type: token_type,
      line: 0,
      value: String::from(""),
      column: 0,
    }
  }

  /// Gets the type of the token instance.
  ///
  /// Must be a value of TokenType enum.
  ///
  /// # Examples
  /// ```
  /// use rust6502lexer::{Token, TokenType};
  /// let token = Token::new(TokenType::CParen).build();
  /// assert_eq!(token.get_type(), &TokenType::CParen);
  /// ```
  pub fn get_type(&self) -> &TokenType {
    &self.token_type
  }

  /// Gets the string value of the token instance.
  ///
  /// # Examples
  /// ```
  /// use rust6502lexer::{Token, TokenType};
  /// let equals = String::from("=");
  /// let token = Token::new(TokenType::Assignment).value(equals).build();
  /// let equals = String::from("=");
  /// assert_eq!(token.get_value(), &equals);
  /// ```
  pub fn get_value(&self) -> &String {
    &self.value
  }

  /// Gets the line number of the token instance.
  ///
  /// # Examples
  /// ```
  /// use rust6502lexer::{Token, TokenType};
  /// let line = 25;
  /// let token = Token::new(TokenType::Label).line(line).build();
  /// let line = 25;
  /// assert_eq!(token.get_line(), &line);
  /// ```
  pub fn get_line(&self) -> &usize {
    &self.line
  }

  /// Gets the line number of the token instance.
  ///
  /// # Examples
  /// ```
  /// use rust6502lexer::{Token, TokenType};
  /// let column = 25;
  /// let token = Token::new(TokenType::Label).column(column).build();
  /// let column = 25;
  /// assert_eq!(token.get_column(), &column);
  /// ```
  pub fn get_column(&self) -> &usize {
    &self.column
  }
}

/// Builder for tokens.
///
/// This mainly exists for readability of lexer code.
///
/// # Examples
/// ```
/// use rust6502lexer::{Token, TokenType};
/// let token_builder = Token::new(TokenType::Identifier);
/// let token = token_builder.line(25).column(25).build();
/// ```
pub struct TokenBuilder {
  token_type: TokenType,
  line: usize,
  value: String,
  column: usize,
}

impl TokenBuilder {
  /// Consume the builder and return a Token instance
  ///
  /// # Examples
  /// ```
  /// use rust6502lexer::{Token, TokenType};
  /// let token_builder = Token::new(TokenType::Identifier);
  /// let token = token_builder.line(25).column(25).build();
  /// ```
  pub fn build(self) -> Token {
    Token {
      token_type: self.token_type,
      line: self.line,
      value: self.value,
      column: self.column,
    }
  }

  /// Add a line to the builder. Defaults to 0
  ///
  /// # Examples
  /// ```
  /// use rust6502lexer::{Token, TokenType};
  /// let token_builder = Token::new(TokenType::Identifier);
  /// let token = token_builder.line(25).column(25).build();
  /// assert_eq!(token.get_line(), &25);
  /// ```
  pub fn line(mut self, line: usize) -> TokenBuilder {
    self.line = line;
    self
  }

  /// Add a line to the builder. Defaults to 0
  ///
  /// # Examples
  /// ```
  /// use rust6502lexer::{Token, TokenType};
  /// let token_builder = Token::new(TokenType::Identifier);
  /// let string_value = String::from("hi");
  /// let token = token_builder.line(25).column(25).value(string_value).build();
  /// assert_eq!(token.get_value(), &String::from("hi"));
  /// ```
  pub fn value(mut self, value: String) -> TokenBuilder {
    self.value = value;
    self
  }

  /// Add a line to the builder. Defaults to 0
  ///
  /// # Examples
  /// ```
  /// use rust6502lexer::{Token, TokenType};
  /// let token_builder = Token::new(TokenType::Identifier);
  /// let token = token_builder.line(25).column(25).build();
  /// assert_eq!(token.get_column(), &25);
  /// ```
  pub fn column(mut self, column: usize) -> TokenBuilder {
    self.column = column;
    self
  }
}

#[derive(Eq, PartialEq, Debug)]
/// The type of the token.
///
/// Provides information to the parser about what we intend.
///
/// # Examples
/// ```
/// use rust6502lexer::{TokenType};
/// let token_type = TokenType::Comma;
/// ```
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
  /// Convenient way to get an operator type from a char
  ///
  /// Checks against a set of stuff and returns the appropriate
  /// token type. Defaults to comma cause I don't wanna wrap this
  /// in an option.
  ///
  /// # Examples
  /// ```
  /// use rust6502lexer::TokenType;
  /// let token_type = TokenType::from_ch('=');
  /// assert_eq!(token_type, TokenType::Assignment);
  /// ```
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new_token() {
    let t1 = Token::new(TokenType::LoByte)
      .value(String::from("hi"))
      .column(25)
      .line(25)
      .build();
    assert_eq!(t1.get_value(), &String::from("hi"));
    assert_eq!(t1.get_column(), &25);
    assert_eq!(t1.get_line(), &25);
    assert_eq!(t1.get_type(), &TokenType::LoByte);
  }

  #[test]
  fn from_ch() {
    let token_type = TokenType::from_ch('>');
    assert_eq!(token_type, TokenType::HiByte);
  }

  #[test]
  fn builder_line() {
    let mut builder = TokenBuilder {
      value: String::from("hi"),
      line: 0,
      column: 0,
      token_type: TokenType::Identifier,
    };
    builder = builder.line(2);
    assert_eq!(builder.line, 2);
  }

  #[test]
  fn builder_column() {
    let mut builder = TokenBuilder {
      value: String::from("hi"),
      line: 0,
      column: 0,
      token_type: TokenType::Identifier,
    };
    builder = builder.column(2);
    assert_eq!(builder.column, 2);
  }

  #[test]
  fn builder_value() {
    let mut builder = TokenBuilder {
      value: String::from("hi"),
      line: 0,
      column: 0,
      token_type: TokenType::Identifier,
    };
    builder = builder.value(String::from("howdy"));
    assert_eq!(builder.value, String::from("howdy"));
  }
}
