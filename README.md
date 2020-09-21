# Rust 6502 Library

A comprehensive 6502 emulator written in Rust.

## Getting started

- `cargo build` to build the lib
- `cargo run program` to run the default program (2 opcodes)
- `cargo run parser` to run the parser (Currently disabled)
- `cargo test` to run the unit tests

## Project goals

1. Have a cycle accurate 6502 emulator that can be exposed as a library for whatever purpose (in progress)
   1. Allow for running cycle by cycle
   1. Allow for running with an external clock simulator
1. Have a 6502 assembler bundled in to facilitate ease of code use
1. Have a 6502 disassembler bundled in for the same reason
1. Have a REPL mode to allow for users to easily debug their assembly
