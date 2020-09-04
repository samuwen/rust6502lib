use emu_attack::*;
use flexi_logger::{opt_format, Logger};
use std::io::{self, stdout, Write};

fn main() {
    Logger::with_env_or_str("trace")
        .log_to_file()
        .directory("log_files")
        .format(opt_format)
        .start()
        .unwrap();
    let mut cpu = CPU::new();
    let mut buffer = String::new();
    loop {
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
                "cld" => cpu.cld(),
                "cli" => cpu.cli(),
                "clv" => cpu.clv(),
                _ => println!("{}", error_message),
            },
            _ => {
                let mode = determine_mode(string[1]);
                let mut iter = string[1].chars();
                match string[0] {
                    "lda" => match mode.as_ref() {
                        "immediate" => cpu.lda(parse_immediate(&mut iter)),
                        _ => (),
                    },
                    "adc" => match mode.as_ref() {
                        "immediate" => cpu.adc(parse_immediate(&mut iter)),
                        "zero page" => cpu.adc_zero_page(parse_zero_page(&mut iter)),
                        "zero page x" => cpu.adc_zero_page_indexed(parse_zero_page_x(&mut iter)),
                        _ => (),
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
    u8::from_str_radix(&digits, 16).unwrap()
}

fn parse_zero_page<'a, I>(iter: &mut I) -> u8
where
    I: Iterator<Item = char>,
{
    // drop the $
    let iter = iter.skip(1);
    let digits: String = iter.collect();
    u8::from_str_radix(&digits, 16).unwrap()
}

fn parse_zero_page_x<'a, I>(iter: &mut I) -> u8
where
    I: Iterator<Item = char>,
{
    // drop the $
    let iter = iter.skip(1);
    let digiterator = iter.take(2);
    let digits: String = digiterator.collect();
    u8::from_str_radix(&digits, 16).unwrap()
}

fn determine_mode(string: &str) -> String {
    let result = match string.len() {
        3 => "zero page",
        4 => "immediate",
        5 => match string.find('X') {
            Some(_) => "zero page x",
            None => "absolute",
        },
        7 => match string.find('(') {
            Some(_) => match string.find('X') {
                Some(_) => "Indirect X",
                None => "Indirect Y",
            },
            None => match string.find('X') {
                Some(_) => "Absolute X",
                None => "Absolute Y",
            },
        },
        _ => "Invalid value",
    };
    String::from(result)
}
