mod memory;
mod registers;

use log::trace;
use memory::Memory;
use registers::{GeneralRegister, ProgramCounter, StackPointer, StatusRegister};
use std::fmt::{Display, Formatter, Result};

/// An emulated CPU for the 6502 processor.
pub struct CPU {
  program_counter: ProgramCounter,
  stack_pointer: StackPointer,
  accumulator: GeneralRegister,
  x_register: GeneralRegister,
  y_register: GeneralRegister,
  status_register: StatusRegister,
  memory: Memory,
}

impl CPU {
  /// Initializes a new CPU instance. Sets all values to 0 by default.
  pub fn new() -> CPU {
    trace!("Initializing CPU");
    CPU {
      program_counter: ProgramCounter::new(),
      stack_pointer: StackPointer::new(),
      accumulator: GeneralRegister::new(),
      x_register: GeneralRegister::new(),
      y_register: GeneralRegister::new(),
      status_register: StatusRegister::new(),
      memory: Memory::new(),
    }
  }

  /// Resets the CPU to its initial state. Zeroes everything out basically.
  pub fn reset(&mut self) {
    self.program_counter.reset();
    self.stack_pointer.reset();
    self.accumulator.reset();
    self.x_register.reset();
    self.y_register.reset();
    self.status_register.reset();
    self.memory.reset();
    trace!("CPU Reset")
  }

  /// Runs a program while there are opcodes to handle. This will change when we actually have
  /// a real data set to operate against.
  pub fn run(&mut self, program: Vec<u8>) {
    while self.program_counter.get() < program.len() {
      let opcode = program[self.program_counter.get()];
      match opcode {
        0x69 => {
          let operand = program[self.program_counter.get() + 1];
          self.adc(operand);
        }
        0xA9 => {
          let operand = program[self.program_counter.get() + 1];
          self.lda(operand);
        }
        _ => (),
      }
    }
  }

  /*
  ============================================================================================
                                  Generic operations
  ============================================================================================
  */

  /// Generically handles zero page retrieval operations and calls a callback when complete
  fn generic_zero_page<F: FnMut(&mut Self, u8)>(&mut self, index: u8, name: &str, cb: &mut F) {
    trace!("{} zero page called with index: 0x{:X}", name, index);
    let value = self.memory.get_zero_page(index);
    cb(self, value);
  }

  /// Same as above but returns a u8
  ///
  /// Name is shorter because I hate rust's multi-line formatting :)
  fn zp_return<F: FnMut(&mut Self, u8) -> u8>(&mut self, idx: u8, name: &str, cb: &mut F) -> u8 {
    trace!("{} zero page called with index: 0x{:X}", name, idx);
    let value = self.memory.get_zero_page(idx);
    cb(self, value)
  }

  fn generic_zero_page_x<F: FnMut(&mut Self, u8)>(&mut self, op: u8, name: &str, cb: &mut F) {
    trace!("{} zero page x called with operand: 0x{:X}", name, op);
    let index = op.wrapping_add(self.x_register.get());
    cb(self, index);
  }

  fn generic_zero_page_y<F: FnMut(&mut Self, u8)>(&mut self, op: u8, name: &str, cb: &mut F) {
    trace!("{} zero page y called with operand: 0x{:X}", name, op);
    let index = op.wrapping_add(self.y_register.get());
    cb(self, index);
  }

  fn generic_absolute<F: FnMut(&mut Self, u8)>(&mut self, index: u16, name: &str, cb: &mut F) {
    trace!("{} absolute called with index: 0x{:X}", name, index);
    let value = self.memory.get_u16(index);
    cb(self, value);
    self.program_counter.advance(1);
  }

  fn abs_return<F: FnMut(&mut Self, u8) -> u8>(&mut self, idx: u16, name: &str, cb: &mut F) -> u8 {
    trace!("{} absolute called with index: 0x{:X}", name, idx);
    let value = self.memory.get_u16(idx);
    self.program_counter.advance(1);
    cb(self, value)
  }

  fn generic_abs_x<F: FnMut(&mut Self, u8)>(&mut self, index: u16, name: &str, cb: &mut F) {
    trace!("{} absolute x called with index: 0x{:X}", name, index);
    let value = self
      .memory
      .get_u16_and_register(index, self.x_register.get());
    cb(self, value);
    self.program_counter.advance(1);
  }

  fn abs_x_ret<F: FnMut(&mut Self, u8) -> u8>(&mut self, index: u16, name: &str, cb: &mut F) -> u8 {
    trace!("{} absolute x called with index: 0x{:X}", name, index);
    let value = self
      .memory
      .get_u16_and_register(index, self.x_register.get());
    self.program_counter.advance(1);
    cb(self, value)
  }

  fn generic_abs_y<F: FnMut(&mut Self, u8)>(&mut self, index: u16, name: &str, cb: &mut F) {
    trace!("{} absolute y called with index: 0x{:X}", name, index);
    let value = self
      .memory
      .get_u16_and_register(index, self.y_register.get());
    cb(self, value);
    self.program_counter.advance(1);
  }

  /// AKA Indexed indirect AKA pre-indexed
  fn generic_indexed_x<F: FnMut(&mut Self, u8)>(&mut self, op: u8, name: &str, cb: &mut F) {
    trace!("{} indexed x called with operand: 0x{:X}", name, op);
    let value = self.memory.get_pre_indexed_data(op, self.x_register.get());
    cb(self, value);
  }

  /// AKA Indirect indexed AKA post-indexed
  fn generic_indexed_y<F: FnMut(&mut Self, u8)>(&mut self, op: u8, name: &str, cb: &mut F) {
    trace!("{} indexed y called with operand: 0x{:X}", name, op);
    let value = self.memory.get_post_indexed_data(op, self.y_register.get());
    cb(self, value);
  }

  fn flag_operation<F: FnMut(&mut StatusRegister)>(&mut self, name: &str, cb: &mut F) {
    trace!("{} called", name);
    cb(&mut self.status_register);
    self.program_counter.advance(1);
  }

  /*
  ============================================================================================
                                  Opcodes
  ============================================================================================
  */

  /// Adds the value given to the accumulator
  ///
  /// Affects flags N V Z C
  pub fn adc(&mut self, value: u8) {
    let message = "ADC";
    trace!("{} called with value: 0x{:X}", message, value);
    let modifier = match self.status_register.is_carry_bit_set() {
      true => 1,
      false => 0,
    };
    let (result, carry) = self.accumulator.get().overflowing_add(value + modifier);
    self.accumulator.set(result);
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_v_flag(result, message, carry);
    self.status_register.handle_z_flag(result, message);
    self.status_register.handle_c_flag(message, carry);
    self.program_counter.advance(2);
  }

  /// Retrieves the value at zero page memory at index provided by the operand and adds it to the accumulator.
  pub fn adc_zero_page(&mut self, index: u8) {
    self.generic_zero_page(index, "ADC", &mut CPU::adc);
  }

  /// Adds the value of the x register to the operand and uses the resulting index to retrieve a value
  /// from zero page memory. Then adds this value to the accumulator.
  pub fn adc_zero_page_x(&mut self, operand: u8) {
    self.generic_zero_page_x(operand, "ADC", &mut CPU::adc_zero_page);
  }

  /// Retrieves the value at regular memory index and adds it to the accumulator.
  pub fn adc_absolute(&mut self, index: u16) {
    self.generic_absolute(index, "ADC", &mut CPU::adc);
  }

  pub fn adc_absolute_x(&mut self, index: u16) {
    self.generic_abs_x(index, "ADC", &mut CPU::adc);
  }

  pub fn adc_absolute_y(&mut self, index: u16) {
    self.generic_abs_y(index, "ADC", &mut CPU::adc);
  }

  pub fn adc_indexed_x(&mut self, operand: u8) {
    self.generic_indexed_x(operand, "ADC", &mut CPU::adc);
  }

  pub fn adc_indexed_y(&mut self, operand: u8) {
    self.generic_indexed_y(operand, "ADC", &mut CPU::adc);
  }

  /// Bitwise and operation performed against the accumulator
  ///
  /// Affects flags N Z
  pub fn and(&mut self, value: u8) {
    let message = "AND";
    trace!("{} called with value: 0x{:X}", message, value);
    let result = self.accumulator.get() & value;
    self.accumulator.set(result);
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_z_flag(result, message);
    self.program_counter.advance(2);
  }

  pub fn and_zero_page(&mut self, index: u8) {
    self.generic_zero_page(index, "AND", &mut CPU::and);
  }

  pub fn and_zero_page_x(&mut self, index: u8) {
    self.generic_zero_page_x(index, "AND", &mut CPU::and_zero_page);
  }

  pub fn and_absolute(&mut self, index: u16) {
    self.generic_absolute(index, "AND", &mut CPU::and);
  }

  pub fn and_absolute_x(&mut self, index: u16) {
    self.generic_abs_x(index, "AND", &mut CPU::and);
  }

  pub fn and_absolute_y(&mut self, index: u16) {
    self.generic_abs_y(index, "AND", &mut CPU::and);
  }

  pub fn and_indexed_x(&mut self, operand: u8) {
    self.generic_indexed_x(operand, "AND", &mut CPU::and);
  }

  pub fn and_indexed_y(&mut self, operand: u8) {
    self.generic_indexed_y(operand, "AND", &mut CPU::and);
  }

  /// Shifts all bits left one position
  ///
  /// Affects flags N Z C
  fn asl(&mut self, value: u8) -> u8 {
    let (result, carry) = value.overflowing_shl(1);
    self.status_register.handle_n_flag(result, "ASL");
    self.status_register.handle_z_flag(result, "ASL");
    self.status_register.handle_c_flag("ASL", carry);
    result
  }

  pub fn asl_accumulator(&mut self) {
    let result = self.asl(self.accumulator.get());
    self.accumulator.set(result);
  }

  pub fn asl_zero_page(&mut self, index: u8) {
    let result = self.zp_return(index, "ASL", &mut CPU::asl);
    self.memory.set_zero_page(index, result);
  }

  pub fn asl_zero_page_x(&mut self, operand: u8) {
    self.generic_zero_page_x(operand, "ASL", &mut CPU::asl_zero_page);
  }

  pub fn asl_absolute(&mut self, index: u16) {
    let result = self.abs_return(index, "ASL", &mut CPU::asl);
    self.memory.set(index, result);
  }

  pub fn asl_absolute_x(&mut self, index: u16) {
    let result = self.abs_x_ret(index, "ASL", &mut CPU::asl);
    let index = index + self.x_register.get() as u16;
    self.memory.set(index, result);
  }

  /// Tests a value and sets flags accordingly.
  ///
  /// Zero is set by looking at the result of the value AND with the accumulator.
  /// N & V are set by bits 7 & 6 of the value respectively.
  ///
  /// Affects flags N V Z
  fn bit(&mut self, value_to_test: u8) {
    let n_result = self.accumulator.get() & value_to_test;
    self.status_register.handle_n_flag(value_to_test, "BIT");
    self.status_register.handle_z_flag(n_result, "BIT");
    if (value_to_test & 0x40) >> 6 == 1 {
      self.status_register.set_overflow_bit();
    } else {
      self.status_register.clear_overflow_bit();
    }
  }

  pub fn bit_zero_page(&mut self, index: u8) {
    self.bit(self.memory.get_zero_page(index));
  }

  pub fn bit_absolute(&mut self, index: u16) {
    self.bit(self.memory.get_u16(index));
  }

  pub fn clc(&mut self) {
    self.flag_operation("CLC", &mut StatusRegister::clear_carry_bit);
  }

  pub fn cld(&mut self) {
    self.flag_operation("CLD", &mut StatusRegister::clear_decimal_bit);
  }

  pub fn cli(&mut self) {
    self.flag_operation("CLI", &mut StatusRegister::clear_interrupt_bit);
  }

  pub fn clv(&mut self) {
    self.flag_operation("CLV", &mut StatusRegister::clear_overflow_bit);
  }

  /// Loads the accumulator with the value given
  ///
  /// Affects flags N Z
  pub fn lda(&mut self, value: u8) {
    let message = "LDA";
    trace!("{} called with value: 0x{:X}", message, value);
    self.accumulator.set(value);
    self.status_register.handle_n_flag(value, message);
    self.status_register.handle_z_flag(value, message);
    self.program_counter.advance(2);
  }

  pub fn lda_zero_page(&mut self, index: u8) {
    self.generic_zero_page(index, "LDA", &mut CPU::lda);
  }

  pub fn lda_zero_page_x(&mut self, operand: u8) {
    self.generic_zero_page_x(operand, "LDA", &mut CPU::lda_zero_page);
  }

  pub fn lda_absolute(&mut self, index: u16) {
    self.generic_absolute(index, "LDA", &mut CPU::lda);
  }

  pub fn lda_absolute_x(&mut self, index: u16) {
    self.generic_abs_x(index, "LDA", &mut CPU::lda);
  }

  pub fn lda_absolute_y(&mut self, index: u16) {
    self.generic_abs_y(index, "LDA", &mut CPU::lda);
  }

  pub fn lda_indexed_x(&mut self, operand: u8) {
    self.generic_indexed_x(operand, "LDA", &mut CPU::lda);
  }

  pub fn lda_indexed_y(&mut self, operand: u8) {
    self.generic_indexed_y(operand, "LDA", &mut CPU::lda);
  }

  /// Loads a value into the X register.
  ///
  /// Flags: N, Z
  pub fn ldx(&mut self, value: u8) {
    let message = "LDX";
    trace!("{} called with value: 0x{:X}", message, value);
    self.x_register.set(value);
    self.status_register.handle_n_flag(value, message);
    self.status_register.handle_z_flag(value, message);
    self.program_counter.advance(2);
  }

  pub fn ldx_zero_page(&mut self, index: u8) {
    self.generic_zero_page(index, "LDX", &mut CPU::ldx);
  }

  pub fn ldx_zero_page_y(&mut self, index: u8) {
    self.generic_zero_page_y(index, "LDX", &mut CPU::ldx_zero_page);
  }

  pub fn ldx_absolute(&mut self, index: u16) {
    self.generic_absolute(index, "LDX", &mut CPU::ldx);
  }

  pub fn ldx_absolute_y(&mut self, index: u16) {
    self.generic_abs_y(index, "LDX", &mut CPU::ldx);
  }

  /// Loads a value into the Y register.
  ///
  /// Flags: N, Z
  pub fn ldy(&mut self, value: u8) {
    let message = "LDY";
    trace!("{} called with value: 0x{:X}", message, value);
    self.y_register.set(value);
    self.status_register.handle_n_flag(value, message);
    self.status_register.handle_z_flag(value, message);
    self.program_counter.advance(2);
  }

  pub fn ldy_zero_page(&mut self, index: u8) {
    self.generic_zero_page(index, "LDY", &mut CPU::ldy);
  }

  pub fn ldy_zero_page_x(&mut self, index: u8) {
    self.generic_zero_page_x(index, "LDY", &mut CPU::ldy_zero_page);
  }

  pub fn ldy_absolute(&mut self, index: u16) {
    self.generic_absolute(index, "LDY", &mut CPU::ldy);
  }

  pub fn ldy_absolute_x(&mut self, index: u16) {
    self.generic_abs_x(index, "LDY", &mut CPU::ldy);
  }

  pub fn sec(&mut self) {
    self.flag_operation("SEC", &mut StatusRegister::set_carry_bit);
  }

  pub fn sed(&mut self) {
    self.flag_operation("SED", &mut StatusRegister::set_decimal_bit);
  }

  pub fn sei(&mut self) {
    self.flag_operation("SEI", &mut StatusRegister::set_interrupt_bit);
  }

  pub fn sta_zero_page(&mut self, index: u8) {
    trace!(
      "STA storing 0x{:X} at 0x{:X}",
      self.accumulator.get(),
      index
    );
    self.memory.set_zero_page(index, self.accumulator.get());
    self.program_counter.advance(2);
  }

  pub fn sta_zero_page_x(&mut self, operand: u8) {
    let index = operand.wrapping_add(self.x_register.get());
    trace!(
      "STA storing 0x{:X} at 0x{:X}",
      self.accumulator.get(),
      index
    );
    self.memory.set_zero_page(index, self.accumulator.get());
    self.program_counter.advance(2);
  }

  pub fn sta_absolute(&mut self, index: u16) {
    trace!(
      "STA storing 0x{:X} at 0x{:X}",
      self.accumulator.get(),
      index
    );
    self.memory.set(index, self.accumulator.get());
    self.program_counter.advance(3);
  }

  pub fn sta_absolute_x(&mut self, index: u16) {
    let index = index + self.x_register.get() as u16;
    trace!(
      "STA storing 0x{:X} at 0x{:X}",
      self.accumulator.get(),
      index
    );
    self.memory.set(index, self.accumulator.get());
    self.program_counter.advance(3);
  }

  pub fn sta_absolute_y(&mut self, index: u16) {
    let index = index + self.y_register.get() as u16;
    trace!(
      "STA storing 0x{:X} at 0x{:X}",
      self.accumulator.get(),
      index
    );
    self.memory.set(index, self.accumulator.get());
    self.program_counter.advance(3);
  }

  pub fn sta_indexed_x(&mut self, operand: u8) {
    let index = self
      .memory
      .get_pre_adjusted_index(operand, self.x_register.get());
    trace!(
      "STA storing 0x{:X} at 0x{:X}",
      self.accumulator.get(),
      index
    );
    self.memory.set(index, self.accumulator.get());
    self.program_counter.advance(2);
  }

  pub fn sta_indexed_y(&mut self, operand: u8) {
    let index = self
      .memory
      .get_post_adjusted_index(operand, self.y_register.get());
    trace!(
      "STA storing 0x{:X} at 0x{:X}",
      self.accumulator.get(),
      index
    );
    self.memory.set(index, self.accumulator.get());
    self.program_counter.advance(2);
  }
}

impl Display for CPU {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(
      f,
      "program_counter: 0x{:X}\nstack_pointer: 0x{:X}\naccumulator: 0x{:X}\nstatus_register: {}\nx_register: 0x{:X}\ny_register: 0x{:X}\n",
      self.program_counter.get(), self.stack_pointer.get(), self.accumulator.get(), self.status_register, self.x_register.get(), self.y_register.get()
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn create_new_cpu() {
    let cpu = CPU::new();
    assert_eq!(cpu.accumulator.get(), 0);
    assert_eq!(cpu.program_counter.get(), 0);
    assert_eq!(cpu.stack_pointer.get(), 0);
    assert_eq!(cpu.x_register.get(), 0);
    assert_eq!(cpu.y_register.get(), 0);
  }

  #[test]
  fn reset_cpu() {
    let mut cpu = CPU::new();
    cpu.stack_pointer.decrement();
    cpu.program_counter.advance(1);
    cpu.accumulator.set(23);
    cpu.x_register.set(23);
    cpu.y_register.set(23);
    cpu.status_register.set_break_bit();
    cpu.reset();
    assert_eq!(cpu.accumulator.get(), 0);
    assert_eq!(cpu.program_counter.get(), 0);
    assert_eq!(cpu.stack_pointer.get(), 0);
    assert_eq!(cpu.x_register.get(), 0);
    assert_eq!(cpu.y_register.get(), 0);
  }

  #[test]
  fn adc_basic() {
    let mut cpu = CPU::new();
    cpu.accumulator.set(0xAA);
    cpu.adc(0x11);
    assert_eq!(cpu.accumulator.get(), 0xAA + 0x11);
  }

  #[test]
  fn adc_with_carry() {
    let mut cpu = CPU::new();
    cpu.accumulator.set(0x00);
    cpu.status_register.set_carry_bit();
    cpu.adc(0x11);
    assert_eq!(cpu.accumulator.get(), 0x12);
  }

  #[test]
  fn adc_zero_page() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x12, 10);
    cpu.accumulator.set(0x32);
    cpu.adc_zero_page(0x12);
    assert_eq!(cpu.accumulator.get(), 0x32 + 10);
  }

  #[test]
  fn adc_zero_page_indexed_no_wrap() {
    let mut cpu = CPU::new();
    cpu.accumulator.set(0x32);
    cpu.x_register.set(0x11);
    cpu.memory.set(0x23, 48);
    cpu.adc_zero_page_x(0x12);
    assert_eq!(cpu.accumulator.get(), 0x32 + 48);
  }

  #[test]
  fn adc_zero_page_indexed_wrap() {
    let mut cpu = CPU::new();
    cpu.accumulator.set(0x32);
    cpu.x_register.set(0x11);
    cpu.memory.set(0x10, 48);
    cpu.adc_zero_page_x(0xFF);
    assert_eq!(cpu.accumulator.get(), 0x32 + 48);
  }

  #[test]
  fn adc_absolute() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x1234, 0x56);
    cpu.accumulator.set(0x10);
    cpu.adc_absolute(0x1234);
    assert_eq!(cpu.accumulator.get(), 0x56 + 0x10);
  }

  #[test]
  fn adc_absolute_x() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x1234 + 0x56, 0x56);
    cpu.x_register.set(0x56);
    cpu.accumulator.set(0x10);
    cpu.adc_absolute_x(0x1234);
    assert_eq!(cpu.accumulator.get(), 0x56 + 0x10);
  }

  #[test]
  fn adc_absolute_y() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x5678 + 0x12, 0x90);
    cpu.y_register.set(0x12);
    cpu.accumulator.set(0xA0);
    cpu.adc_absolute_y(0x5678);
    assert_eq!(cpu.accumulator.get(), 0x30);
  }

  #[test]
  fn sta_zero_page() {
    let mut cpu = CPU::new();
    cpu.accumulator.set(0x34);
    cpu.sta_zero_page(0x55);
    assert_eq!(cpu.memory.get_zero_page(0x55), 0x34);
  }

  #[test]
  fn sta_zero_page_x() {
    let mut cpu = CPU::new();
    cpu.accumulator.set(0x99);
    cpu.x_register.set(0x34);
    cpu.sta_zero_page_x(0x55);
    assert_eq!(cpu.memory.get_zero_page(0x55 + 0x34), 0x99);
  }

  #[test]
  fn sta_absolute() {
    let mut cpu = CPU::new();
    cpu.accumulator.set(0x99);
    cpu.sta_absolute(0x1255);
    assert_eq!(cpu.memory.get_u16(0x1255), 0x99);
  }

  #[test]
  fn sta_absolute_x() {
    let mut cpu = CPU::new();
    cpu.accumulator.set(0x99);
    cpu.x_register.set(0x10);
    cpu.sta_absolute_x(0x1255);
    assert_eq!(cpu.memory.get_u16(0x1265), 0x99);
  }

  #[test]
  fn sta_absolute_y() {
    let mut cpu = CPU::new();
    cpu.accumulator.set(0x99);
    cpu.y_register.set(0x20);
    cpu.sta_absolute_y(0x1275);
    assert_eq!(cpu.memory.get_u16(0x1295), 0x99);
  }

  #[test]
  fn sta_indexed_x() {
    let mut cpu = CPU::new();
    cpu.accumulator.set(0x99);
    cpu.x_register.set(0x20);
    cpu.memory.set(0x32, 0x20);
    cpu.memory.set(0x33, 0x25);
    cpu.sta_indexed_x(0x12);
    assert_eq!(cpu.memory.get_u16(0x2520), 0x99);
  }

  #[test]
  fn sta_indexed_y() {
    let mut cpu = CPU::new();
    cpu.accumulator.set(0x45);
    cpu.y_register.set(0x20);
    cpu.memory.set(0x12, 0x57);
    cpu.memory.set(0x13, 0xCB);
    cpu.sta_indexed_y(0x12);
    assert_eq!(cpu.memory.get_u16(0xCB77), 0x45);
  }

  #[test]
  fn lda() {
    let mut cpu = CPU::new();
    cpu.lda(0x11);
    assert_eq!(cpu.accumulator.get(), 0x11);
  }

  #[test]
  fn lda_zero_page() {
    let mut cpu = CPU::new();
    cpu.memory.set_zero_page(0x32, 0x12);
    cpu.lda_zero_page(0x32);
    assert_eq!(cpu.accumulator.get(), 0x12);
  }

  #[test]
  fn lda_zero_page_x() {
    let mut cpu = CPU::new();
    cpu.x_register.set(0x75);
    cpu.memory.set_zero_page(0x32 + 0x75, 0x12);
    cpu.lda_zero_page_x(0x32);
    assert_eq!(cpu.accumulator.get(), 0x12);
  }

  #[test]
  fn lda_absolute() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x1234, 0x56);
    cpu.lda_absolute(0x1234);
    assert_eq!(cpu.accumulator.get(), 0x56);
  }

  #[test]
  fn lda_absolute_x() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x1254, 0x56);
    cpu.x_register.set(0x20);
    cpu.lda_absolute_x(0x1234);
    assert_eq!(cpu.accumulator.get(), 0x56);
  }

  #[test]
  fn lda_absolute_y() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x1254, 0x56);
    cpu.y_register.set(0x20);
    cpu.lda_absolute_y(0x1234);
    assert_eq!(cpu.accumulator.get(), 0x56);
  }

  #[test]
  fn ldx() {
    let mut cpu = CPU::new();
    cpu.ldx(0x11);
    assert_eq!(cpu.x_register.get(), 0x11);
  }

  #[test]
  fn ldx_zero_page() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x39, 0x11);
    cpu.ldx_zero_page(0x39);
    assert_eq!(cpu.x_register.get(), 0x11);
  }

  #[test]
  fn ldx_zero_page_y() {
    let mut cpu = CPU::new();
    cpu.y_register.set(0x75);
    cpu.memory.set_zero_page(0x32 + 0x75, 0x12);
    cpu.ldx_zero_page_y(0x32);
    assert_eq!(cpu.x_register.get(), 0x12);
  }

  #[test]
  fn ldx_absolute() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x1234, 0x56);
    cpu.ldx_absolute(0x1234);
    assert_eq!(cpu.x_register.get(), 0x56);
  }

  #[test]
  fn ldx_absolute_y() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x1254, 0x56);
    cpu.y_register.set(0x20);
    cpu.ldx_absolute_y(0x1234);
    assert_eq!(cpu.x_register.get(), 0x56);
  }

  #[test]
  fn ldy() {
    let mut cpu = CPU::new();
    cpu.ldy(0x11);
    assert_eq!(cpu.y_register.get(), 0x11);
  }

  #[test]
  fn ldy_zero_page() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x39, 0x11);
    cpu.ldy_zero_page(0x39);
    assert_eq!(cpu.y_register.get(), 0x11);
  }

  #[test]
  fn ldy_zero_page_x() {
    let mut cpu = CPU::new();
    cpu.x_register.set(0x75);
    cpu.memory.set_zero_page(0x32 + 0x75, 0x12);
    cpu.ldy_zero_page_x(0x32);
    assert_eq!(cpu.y_register.get(), 0x12);
  }

  #[test]
  fn ldy_absolute() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x1234, 0x56);
    cpu.ldy_absolute(0x1234);
    assert_eq!(cpu.y_register.get(), 0x56);
  }

  #[test]
  fn ldy_absolute_x() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x1254, 0x56);
    cpu.x_register.set(0x20);
    cpu.ldy_absolute_x(0x1234);
    assert_eq!(cpu.y_register.get(), 0x56);
  }

  #[test]
  fn clc() {
    let mut cpu = CPU::new();
    cpu.status_register.set_carry_bit();
    cpu.clc();
    assert_eq!(cpu.status_register.is_carry_bit_set(), false);
  }

  #[test]
  fn sec() {
    let mut cpu = CPU::new();
    cpu.sec();
    assert_eq!(cpu.status_register.is_carry_bit_set(), true);
  }

  #[test]
  fn clv() {
    let mut cpu = CPU::new();
    cpu.status_register.set_overflow_bit();
    cpu.clv();
    assert_eq!(cpu.status_register.is_overflow_bit_set(), false);
  }

  #[test]
  fn cld() {
    let mut cpu = CPU::new();
    cpu.status_register.set_decimal_bit();
    cpu.cld();
    assert_eq!(cpu.status_register.is_decimal_bit_set(), false);
  }

  #[test]
  fn sed() {
    let mut cpu = CPU::new();
    cpu.sed();
    assert_eq!(cpu.status_register.is_decimal_bit_set(), true);
  }

  #[test]
  fn cli() {
    let mut cpu = CPU::new();
    cpu.status_register.set_interrupt_bit();
    cpu.cli();
    assert_eq!(cpu.status_register.is_interrupt_bit_set(), false);
  }

  #[test]
  fn sei() {
    let mut cpu = CPU::new();
    cpu.sei();
    assert_eq!(cpu.status_register.is_interrupt_bit_set(), true);
  }

  #[test]
  fn and() {
    let mut cpu = CPU::new();
    cpu.accumulator.set(0x69);
    cpu.and(0x57);
    assert_eq!(cpu.accumulator.get(), 0x41);
  }

  #[test]
  fn and_zero_page() {
    let mut cpu = CPU::new();
    cpu.accumulator.set(0x69);
    cpu.memory.set_zero_page(0x57, 0xAC);
    cpu.and_zero_page(0x57);
    assert_eq!(cpu.accumulator.get(), 0x28);
  }

  #[test]
  fn and_zero_page_x() {
    let mut cpu = CPU::new();
    cpu.accumulator.set(0x69);
    cpu.x_register.set(0x08);
    cpu.memory.set_zero_page(0x5F, 0xAC);
    cpu.and_zero_page_x(0x57);
    assert_eq!(cpu.accumulator.get(), 0x28);
  }

  #[test]
  fn and_absolute() {
    let mut cpu = CPU::new();
    cpu.accumulator.set(0x34);
    cpu.memory.set(0x1234, 0x56);
    cpu.and_absolute(0x1234);
    assert_eq!(cpu.accumulator.get(), 0x14);
  }

  #[test]
  fn and_absolute_x() {
    let mut cpu = CPU::new();
    cpu.accumulator.set(0x34);
    cpu.memory.set(0x1254, 0x56);
    cpu.x_register.set(0x20);
    cpu.and_absolute_x(0x1234);
    assert_eq!(cpu.accumulator.get(), 0x14);
  }

  #[test]
  fn and_absolute_y() {
    let mut cpu = CPU::new();
    cpu.accumulator.set(0x34);
    cpu.memory.set(0x1254, 0x56);
    cpu.y_register.set(0x20);
    cpu.and_absolute_y(0x1234);
    assert_eq!(cpu.accumulator.get(), 0x14);
  }

  #[test]
  fn asl() {
    let mut cpu = CPU::new();
    let result = cpu.asl(0xFF);
    assert_eq!(result, 0xFE);
  }

  #[test]
  fn asl_accumulator() {
    let mut cpu = CPU::new();
    cpu.accumulator.set(0x65);
    cpu.asl_accumulator();
    assert_eq!(cpu.accumulator.get(), 0xCA);
  }

  #[test]
  fn asl_zero_page() {
    let mut cpu = CPU::new();
    cpu.memory.set_zero_page(0xBA, 0xCC);
    cpu.asl_zero_page(0xBA);
    assert_eq!(cpu.memory.get_zero_page(0xBA), 0x98);
  }

  #[test]
  fn asl_zero_page_x() {
    let mut cpu = CPU::new();
    cpu.memory.set_zero_page(0x9B, 0xCC);
    cpu.x_register.set(0xE1);
    cpu.asl_zero_page_x(0xBA);
    assert_eq!(cpu.memory.get_zero_page(0x9B), 0x98);
  }

  #[test]
  fn asl_absolute() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x9BB9, 0xDD);
    cpu.asl_absolute(0x9BB9);
    assert_eq!(cpu.memory.get_u16(0x9BB9), 0xBA);
  }

  #[test]
  fn asl_absolute_x() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x9BF1, 0xDD);
    cpu.x_register.set(0x38);
    cpu.asl_absolute_x(0x9BB9);
    assert_eq!(cpu.memory.get_u16(0x9BF1), 0xBA);
  }

  #[test]
  fn bit_zero_bit() {
    let mut cpu = CPU::new();
    cpu.bit(0);
    assert_eq!(cpu.status_register.is_zero_bit_set(), true);
    cpu.accumulator.set(0xFF);
    cpu.bit(0xFF);
    assert_eq!(cpu.status_register.is_zero_bit_set(), false);
  }

  #[test]
  fn bit_overflow_bit() {
    let mut cpu = CPU::new();
    cpu.bit(0);
    assert_eq!(cpu.status_register.is_overflow_bit_set(), false);
    cpu.bit(0xFF);
    assert_eq!(cpu.status_register.is_overflow_bit_set(), true);
  }

  #[test]
  fn bit_negative_bit() {
    let mut cpu = CPU::new();
    cpu.bit(0);
    assert_eq!(cpu.status_register.is_negative_bit_set(), false);
    cpu.bit(0xFF);
    assert_eq!(cpu.status_register.is_negative_bit_set(), true);
  }

  #[test]
  fn bit_zero_page() {
    let mut cpu = CPU::new();
    cpu.memory.set_zero_page(0x65, 0x55);
    cpu.accumulator.set(0xAA);
    cpu.bit_zero_page(0x65);
    assert_eq!(cpu.status_register.is_zero_bit_set(), true);
    assert_eq!(cpu.status_register.is_negative_bit_set(), false);
    assert_eq!(cpu.status_register.is_overflow_bit_set(), true);
  }

  #[test]
  fn bit_absolute() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x6556, 0x55);
    cpu.accumulator.set(0xAA);
    cpu.bit_absolute(0x6556);
    assert_eq!(cpu.status_register.is_zero_bit_set(), true);
    assert_eq!(cpu.status_register.is_negative_bit_set(), false);
    assert_eq!(cpu.status_register.is_overflow_bit_set(), true);
  }
}
