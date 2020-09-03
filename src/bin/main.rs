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
        match string[0] {
            "exit" => break,
            "lda" => {
                let mut iter = string[1].chars();
                match iter.next().unwrap() {
                    '#' => {
                        let parsed = parse_immediate(&mut iter);
                        cpu.lda(parsed);
                    }
                    _ => (),
                }
            }
            "print" => println!("{}", cpu),
            "reset" => cpu.reset(),
            _ => println!("Invalid input. Try another command"),
        }
        buffer = String::new();
    }
}

fn parse_immediate<'a, I>(iter: &mut I) -> u8
where
    I: Iterator<Item = char>,
{
    iter.next().unwrap();
    let digits: String = iter.collect();
    u8::from_str_radix(&digits, 16).unwrap()
}
