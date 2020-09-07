use emu_attack::CPU;
use std::io::{self, stdout, Write};

const BASE_RADIX: u32 = 16;

pub struct Parser {
  cpu: CPU,
}

impl Parser {
  pub fn new() -> Parser {
    Parser { cpu: CPU::new() }
  }

  pub fn run(&mut self) {
    let mut buffer = String::new();
    loop {
      let cpu = &mut self.cpu;
      print!("> ");
      stdout().flush().unwrap();
      let stdin = io::stdin();
      stdin
        .read_line(&mut buffer)
        .expect("Failed to capture input");
      buffer.make_ascii_lowercase();
      let string: Vec<_> = buffer.trim().split(' ').collect();
      let error_message = "Invalid input. Try another command";
      match string.len() {
        0 => println!("{}", error_message),
        1 => match string[0] {
          "print" => println!("{}", cpu),
          "reset" => cpu.reset(),
          "exit" => break,
          "clc" => cpu.clc(),
          "sec" => cpu.sec(),
          "cld" => cpu.cld(),
          "cli" => cpu.cli(),
          "sei" => cpu.sei(),
          "clv" => cpu.clv(),
          _ => println!("{}", error_message),
        },
        _ => {
          let mode = Parser::determine_mode(string[1]);
          let mut iter = string[1].chars();
          match string[0] {
            "lda" => match mode {
              AddressMode::Immediate => cpu.lda(Parser::parse_immediate(&mut iter)),
              AddressMode::ZeroPage => cpu.lda_zero_page(Parser::parse_zero_page(&mut iter)),
              AddressMode::ZeroPageX => cpu.lda_zero_page_x(Parser::parse_zero_page_x(&mut iter)),
              AddressMode::Absolute => cpu.lda_absolute(Parser::parse_absolute(&mut iter)),
              AddressMode::AbsoluteX => cpu.lda_absolute_x(Parser::parse_absolute(&mut iter)),
              AddressMode::AbsoluteY => cpu.lda_absolute_y(Parser::parse_absolute(&mut iter)),
              AddressMode::IndirectX => cpu.lda_indexed_x(Parser::parse_immediate(&mut iter)),
              AddressMode::IndirectY => cpu.lda_indexed_y(Parser::parse_immediate(&mut iter)),
              AddressMode::Invalid => println!("Invalid LDA request syntax"),
              _ => (),
            },
            "ldx" => match mode {
              AddressMode::Immediate => cpu.ldx(Parser::parse_immediate(&mut iter)),
              AddressMode::ZeroPage => cpu.ldx_zero_page(Parser::parse_zero_page(&mut iter)),
              AddressMode::ZeroPageY => cpu.ldx_zero_page_y(Parser::parse_zero_page_x(&mut iter)),
              AddressMode::Absolute => cpu.ldx_absolute(Parser::parse_absolute(&mut iter)),
              AddressMode::AbsoluteY => cpu.ldx_absolute_y(Parser::parse_absolute(&mut iter)),
              _ => println!("Invalid LDX request syntax"),
            },
            "ldy" => match mode {
              AddressMode::Immediate => cpu.ldy(Parser::parse_immediate(&mut iter)),
              AddressMode::ZeroPage => cpu.ldy_zero_page(Parser::parse_zero_page(&mut iter)),
              AddressMode::ZeroPageX => cpu.ldy_zero_page_x(Parser::parse_zero_page_x(&mut iter)),
              AddressMode::Absolute => cpu.ldy_absolute(Parser::parse_absolute(&mut iter)),
              AddressMode::AbsoluteX => cpu.ldy_absolute_x(Parser::parse_absolute(&mut iter)),
              _ => println!("Invalid LDY request syntax"),
            },
            "adc" => match mode {
              AddressMode::Immediate => cpu.adc(Parser::parse_immediate(&mut iter)),
              AddressMode::ZeroPage => cpu.adc_zero_page(Parser::parse_zero_page(&mut iter)),
              AddressMode::ZeroPageX => cpu.adc_zero_page_x(Parser::parse_zero_page_x(&mut iter)),
              AddressMode::Absolute => cpu.adc_absolute(Parser::parse_absolute(&mut iter)),
              AddressMode::AbsoluteX => cpu.adc_absolute_x(Parser::parse_absolute(&mut iter)),
              AddressMode::AbsoluteY => cpu.adc_absolute_y(Parser::parse_absolute(&mut iter)),
              AddressMode::IndirectX => cpu.adc_indexed_x(Parser::parse_immediate(&mut iter)),
              AddressMode::IndirectY => cpu.adc_indexed_y(Parser::parse_immediate(&mut iter)),
              _ => println!("Invalid ADC request syntax"),
            },
            "and" => match mode {
              AddressMode::Immediate => cpu.and(Parser::parse_immediate(&mut iter)),
              AddressMode::ZeroPage => cpu.and_zero_page(Parser::parse_zero_page(&mut iter)),
              AddressMode::ZeroPageX => cpu.and_zero_page_x(Parser::parse_zero_page_x(&mut iter)),
              AddressMode::Absolute => cpu.and_absolute(Parser::parse_absolute(&mut iter)),
              AddressMode::AbsoluteX => cpu.and_absolute_x(Parser::parse_absolute(&mut iter)),
              AddressMode::AbsoluteY => cpu.and_absolute_y(Parser::parse_absolute(&mut iter)),
              AddressMode::IndirectX => cpu.and_indexed_x(Parser::parse_immediate(&mut iter)),
              AddressMode::IndirectY => cpu.and_indexed_y(Parser::parse_immediate(&mut iter)),
              _ => println!("Invalid AND request syntax"),
            },
            "sta" => match mode {
              AddressMode::ZeroPage => cpu.sta_zero_page(Parser::parse_zero_page(&mut iter)),
              AddressMode::ZeroPageX => cpu.sta_zero_page_x(Parser::parse_zero_page_x(&mut iter)),
              AddressMode::Absolute => cpu.sta_absolute(Parser::parse_absolute(&mut iter)),
              AddressMode::AbsoluteX => cpu.sta_absolute_x(Parser::parse_absolute(&mut iter)),
              AddressMode::AbsoluteY => cpu.sta_absolute_y(Parser::parse_absolute(&mut iter)),
              AddressMode::IndirectX => cpu.sta_indexed_x(Parser::parse_immediate(&mut iter)),
              AddressMode::IndirectY => cpu.sta_indexed_y(Parser::parse_immediate(&mut iter)),
              _ => println!("Invalid STA request syntax"),
            },
            _ => println!("{}", error_message),
          }
        }
      }
      buffer = String::new();
    }
  }

  fn parse_immediate<'a, I>(iter: &mut I) -> u8
  where
    I: Iterator<Item = char>,
  {
    // drop the #$
    let iter = iter.skip(2);
    let digits: String = iter.collect();
    u8::from_str_radix(&digits, BASE_RADIX).unwrap()
  }

  fn parse_zero_page<'a, I>(iter: &mut I) -> u8
  where
    I: Iterator<Item = char>,
  {
    // drop the $
    let iter = iter.skip(1);
    let digits: String = iter.collect();
    u8::from_str_radix(&digits, BASE_RADIX).unwrap()
  }

  fn parse_zero_page_x<'a, I>(iter: &mut I) -> u8
  where
    I: Iterator<Item = char>,
  {
    // drop the $
    let iter = iter.skip(1);
    let digiterator = iter.take(2);
    let digits: String = digiterator.collect();
    u8::from_str_radix(&digits, BASE_RADIX).unwrap()
  }

  fn parse_absolute<'a, I>(iter: &mut I) -> u16
  where
    I: Iterator<Item = char>,
  {
    let iter = iter.skip(1);
    let mut first_two = String::new();
    let mut last_two = String::new();
    for (i, cha) in iter.enumerate() {
      if i < 2 {
        first_two.push(cha);
      } else {
        last_two.push(cha);
      }
    }
    last_two.push_str(first_two.as_ref());
    u16::from_str_radix(&last_two, BASE_RADIX).unwrap()
  }

  fn determine_mode(string: &str) -> AddressMode {
    match string.len() {
      3 => AddressMode::ZeroPage,
      4 => AddressMode::Immediate,
      5 => match string.find('X') {
        Some(_) => AddressMode::ZeroPageX,
        None => match string.find('Y') {
          Some(_) => AddressMode::ZeroPageY,
          None => AddressMode::Absolute,
        },
      },
      7 => match string.find('(') {
        Some(_) => match string.find('X') {
          Some(_) => AddressMode::IndirectX,
          None => AddressMode::IndirectY,
        },
        None => match string.find('X') {
          Some(_) => AddressMode::AbsoluteX,
          None => AddressMode::AbsoluteY,
        },
      },
      _ => AddressMode::Invalid,
    }
  }
}

enum AddressMode {
  Immediate,
  ZeroPage,
  ZeroPageX,
  ZeroPageY,
  Absolute,
  AbsoluteX,
  AbsoluteY,
  IndirectX,
  IndirectY,
  Invalid,
}
