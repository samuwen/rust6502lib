use crate::utils::char_utils::*;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
pub struct Token {
  token_type: TokenType,
  line: usize,
  value: String,
  column: usize,
}

impl Token {
  fn new(token_type: TokenType) -> TokenBuilder {
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
}

struct TokenBuilder {
  token_type: TokenType,
  line: usize,
  value: String,
  column: usize,
}

impl TokenBuilder {
  fn build(self) -> Token {
    Token {
      token_type: self.token_type,
      line: self.line,
      value: self.value,
      column: self.column,
    }
  }

  fn line(mut self, line: usize) -> TokenBuilder {
    self.line = line;
    self
  }

  fn value(mut self, value: String) -> TokenBuilder {
    self.value = value;
    self
  }

  fn column(mut self, column: usize) -> TokenBuilder {
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
  fn from_ch(ch: char) -> TokenType {
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

pub struct Lexer<'l> {
  text: Box<Peekable<Chars<'l>>>,
  line: usize,
  column: usize,
}

impl<'l> Lexer<'l> {
  pub fn new(input: Box<Peekable<Chars<'l>>>) -> Lexer<'l> {
    Lexer {
      text: input,
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
