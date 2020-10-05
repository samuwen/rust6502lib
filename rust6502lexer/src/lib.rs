mod char_utils;
pub mod token;

use char_utils::*;
use std::iter::Peekable;
use std::str::Chars;
pub use token::{Token, TokenType};

/// A 6502 ca65 Lexer
///
/// Takes in a string and will output Token objects containing
/// line info, column info, and token type info.
///
/// # Example
/// Basic usage:
/// ```
/// use rust6502lexer::{Lexer, TokenType};
///
/// let test_string = String::from("a + b");
/// let mut lexer = Lexer::new(&test_string);
/// let token = lexer.next_token();
/// assert_eq!(token.get_type(), &TokenType::Identifier);
/// assert_eq!(token.get_value(), "a");
/// ```
pub struct Lexer<'l> {
  text: Box<Peekable<Chars<'l>>>,
  line: usize,
  column: usize,
}

impl<'l> Lexer<'l> {
  /// Creates a new lexer instance with the string to be lexed.
  ///
  /// Accepts a string and stores it as an iterator so that it is
  /// easy to invoke the next token. Starts everything at zero.
  ///
  /// # Examples
  /// ```
  /// use rust6502lexer::Lexer;
  ///
  /// let lexer = Lexer::new(&String::from("hi"));
  /// ```
  pub fn new(input: &'l String) -> Lexer<'l> {
    let text = Box::new(input.chars().peekable());
    Lexer {
      text: text,
      line: 0,
      column: 0,
    }
  }

  /// Processes and returns the next token in the text
  ///
  /// The main meat of this class. This reads in a character and then
  /// generates a token from it, keeping track of position and line number.
  /// All whitespace and newlines are discarded by this method. This will
  /// return an EndOfInput token when it reaches the end of the input.
  ///
  /// # Examples
  /// ```
  /// use rust6502lexer::{Lexer, TokenType};
  ///
  /// let s = String::from("hi");
  /// let mut lexer = Lexer::new(&s);
  /// let t1 = lexer.next_token();
  /// let t2 = lexer.next_token();
  /// assert_eq!(t1.get_type(), &TokenType::Identifier);
  /// assert_eq!(t2.get_type(), &TokenType::EndOfInput);
  /// ```
  pub fn next_token(&mut self) -> Token {
    self.remove_irrelevant();
    let c = self.text.peek();
    if let Some(ch) = c {
      if is_letter(ch) {
        return self.handle_identifier();
      }
      if is_operator(ch) {
        return self.handle_operator();
      }
      if is_number(ch) {
        return self.handle_number();
      }
      if is_decimal(ch) {
        return self.handle_asm_command();
      }
      if is_string(ch) {
        return self.handle_string();
      }
    }
    Token::new(TokenType::EndOfInput).build()
  }

  /// Gets all elements of an identifier and returns the token
  ///
  /// Identifiers are classified as most string values. Mnemonics
  /// or assignments, for example. This method also processes labels
  /// by finding if a colon is present.
  fn handle_identifier(&mut self) -> Token {
    let mut identifier = String::from("");
    let start_char = self.get_next_char().unwrap();
    identifier.push(start_char);
    let mut token_type = TokenType::Identifier;
    loop {
      let c = self.text.peek();
      if let Some(ch) = c {
        match ch {
          ' ' | '\t' | '\n' | '\r' => break,
          ':' => {
            // get the colon and discard it
            self.get_next_char().unwrap();
            token_type = TokenType::Label;
            break;
          }
          _ => {
            let ch = self.get_next_char().unwrap();
            identifier.push(ch);
          }
        }
      } else {
        break;
      }
    }
    self.build_new_token(token_type, identifier)
  }

  /// Gets an operator token and returns it
  fn handle_operator(&mut self) -> Token {
    let mut operator = String::from("");
    let ch = self.get_next_char().unwrap();
    operator.push(ch);
    let token_type = TokenType::from_ch(ch);
    self.build_new_token(token_type, operator)
  }

  /// Gets all elements of a number and returns the token
  ///
  /// Numbers are not parsed for radix, but are bundled with their
  /// identifying operator.
  ///
  /// # Example
  /// ```
  /// let decimal = "2";
  /// let hex = "$2";
  /// let binary = "%00000010";
  /// ```
  fn handle_number(&mut self) -> Token {
    let mut number = String::from("");
    let ch = self.get_next_char().unwrap();
    number.push(ch);
    loop {
      let peek = self.text.peek();
      if let Some(c) = peek {
        match c {
          ',' | ' ' | '\t' | '\n' => break,
          _ => {
            let ch = self.get_next_char().unwrap();
            number.push(ch);
          }
        }
      } else {
        break;
      }
    }
    self.build_new_token(TokenType::Number, number)
  }

  /// Gets all elements of an assembler command
  ///
  /// Currently this is built only for the ca65 assembler, and uses
  /// its syntax.
  fn handle_asm_command(&mut self) -> Token {
    let mut command = String::from("");
    self.get_next_char().unwrap();
    loop {
      let peek = self.text.peek();
      if let Some(c) = peek {
        match c {
          ' ' | '\t' => break,
          _ => {
            let ch = self.get_next_char().unwrap();
            command.push(ch);
          }
        }
      } else {
        break;
      }
    }
    self.build_new_token(TokenType::Command, command)
  }

  /// Gets all elements of a string literal
  fn handle_string(&mut self) -> Token {
    let mut string = String::from("");
    self.get_next_char().unwrap();
    loop {
      let peek = self.text.peek();
      if let Some(c) = peek {
        match c {
          '\'' | '"' => {
            self.get_next_char().unwrap();
            break;
          }
          _ => {
            let ch = self.get_next_char().unwrap();
            string.push(ch);
          }
        }
      } else {
        break;
      }
    }
    self.build_new_token(TokenType::String, string)
  }

  /// Wrapper around the next iterator. Keeps our column value up to date.
  fn get_next_char(&mut self) -> Option<char> {
    self.column += 1;
    self.text.next()
  }

  /// Convenience method to build new tokens. Wraps the token builder.
  fn build_new_token(&mut self, t: TokenType, val: String) -> Token {
    Token::new(t)
      .column(self.column - val.len())
      .line(self.line)
      .value(val)
      .build()
  }

  /// Removes all whitespace and newlines from the text.
  ///
  /// Determines if the next char is something we care about and returns
  /// control back to the main loop if so, otherwise eliminates items and
  /// keeps track of position and column.
  fn remove_irrelevant(&mut self) {
    let mut peek;
    loop {
      peek = self.text.peek();
      if let Some(c) = peek {
        match c {
          ';' => {
            self.handle_comment();
          }
          '\n' | '\r' => {
            self.handle_newline();
            self.get_next_char();
          }
          ' ' | '\t' => {
            self.handle_whitespace();
          }
          _ => {
            break;
          }
        }
      } else {
        break;
      }
    }
  }

  /// If a comment is found, loops until the newline is found, increments and returns.
  fn handle_comment(&mut self) {
    loop {
      let c = self.get_next_char();
      match c {
        Some(c) => match c {
          '\n' => {
            self.handle_newline();
            break;
          }
          _ => (),
        },
        None => break,
      }
    }
  }

  /// Resets the column counter and increments the line counter
  fn handle_newline(&mut self) {
    self.line += 1;
    self.column = 0;
  }

  /// Wrapper around our iterator advancer. Just makes the code more readable.
  fn handle_whitespace(&mut self) {
    self.get_next_char();
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_case::test_case;

  #[test]
  fn new_lexer() {
    let string = String::from("hi");
    let lexer = Lexer::new(&string);
    assert_eq!(lexer.line, 0);
    assert_eq!(lexer.column, 0);
  }

  #[test_case("ADC", TokenType::Identifier)]
  #[test_case("=", TokenType::Assignment)]
  #[test_case(".segment", TokenType::Command)]
  #[test_case("$4400", TokenType::Number)]
  #[test_case("\"HEADER\"", TokenType::String)]
  fn next_token(string: &str, token_type: TokenType) {
    let text = String::from(string);
    let mut lexer = Lexer::new(&text);
    let t1 = lexer.next_token();
    assert_eq!(t1.get_type(), &token_type);
    // We do some pruning in lexing.
    assert_eq!(*t1.get_value(), string.replace("\"", "").replace(".", ""));
  }
}
