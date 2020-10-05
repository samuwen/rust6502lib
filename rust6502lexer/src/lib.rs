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

  fn handle_operator(&mut self) -> Token {
    let mut operator = String::from("");
    let ch = self.get_next_char().unwrap();
    operator.push(ch);
    let token_type = TokenType::from_ch(ch);
    self.build_new_token(token_type, operator)
  }

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

  fn get_next_char(&mut self) -> Option<char> {
    self.column += 1;
    self.text.next()
  }

  fn build_new_token(&mut self, t: TokenType, val: String) -> Token {
    Token::new(t)
      .column(self.column - val.len())
      .line(self.line)
      .value(val)
      .build()
  }

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

  fn handle_newline(&mut self) {
    self.line += 1;
    self.column = 0;
  }

  fn handle_whitespace(&mut self) {
    self.get_next_char();
  }
}
