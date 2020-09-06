mod memory;
mod registers;

use log::trace;
use memory::Memory;
use registers::status_register::StatusRegister;
use std::fmt::{Display, Formatter, Result};

pub struct CPU {
  program_counter: u16,
  stack_pointer: u8,
  accumulator: u8,
  x_register: u8,
  y_register: u8,
  status_register: StatusRegister,
  memory: Memory,
}

impl CPU {
  pub fn new() -> CPU {
    trace!("Initializing CPU");
    CPU {
      program_counter: 0,
      stack_pointer: 0,
      accumulator: 0,
      x_register: 0,
      y_register: 0,
      status_register: StatusRegister::new(),
      memory: Memory::new(),
    }
  }

  pub fn reset(&mut self) {
    self.program_counter = 0;
    self.stack_pointer = 0;
    self.accumulator = 0;
    self.x_register = 0;
    self.y_register = 0;
    self.status_register.reset();
    self.memory.reset();
    trace!("CPU Reset")
  }

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
    let (result, carry) = self.accumulator.overflowing_add(value + modifier);
    self.accumulator = result;
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_v_flag(result, message, carry);
    self.status_register.handle_z_flag(result, message);
    self.status_register.handle_c_flag(message, carry);
  }

  pub fn adc_zero_page(&mut self, index: u8) {
    trace!("ADC zero page calld with index: 0x{:X}", index);
    let value = self.memory.get_zero_page(index);
    self.adc(value);
  }

  pub fn adc_zero_page_indexed(&mut self, operand: u8) {
    trace!("ADC zero page indexed called with operand: 0x{:X}", operand);
    let index = operand.wrapping_add(self.x_register);
    self.adc_zero_page(index);
  }

  pub fn adc_absolute(&mut self, index: u16) {
    trace!("ADC absolute called with index: 0x{:X}", index);
    let value = self.memory.get_u16(index);
    self.adc(value);
  }

  pub fn adc_absolute_x(&mut self, index: u16) {
    trace!("ADC absolute x called with index: 0x{:X}", index);
    let value = self.memory.get_u16_and_register(index, self.x_register);
    self.adc(value);
  }

  pub fn adc_absolute_y(&mut self, index: u16) {
    trace!("ADC absolute y called with index: 0x{:X}", index);
    let value = self.memory.get_u16_and_register(index, self.y_register);
    self.adc(value);
  }

  /// AKA Indexed indirect AKA pre-indexed
  pub fn adc_indexed_x(&mut self, operand: u8) {
    trace!("ADC indexed x called with operand: 0x{:X}", operand);
    let value = self.memory.get_pre_indexed_data(operand, self.x_register);
    self.adc(value);
  }

  /// AKA Indirect indexed AKA post-indexed
  pub fn adc_indexed_y(&mut self, operand: u8) {
    trace!("ADC indexed y called with operand: 0x{:X}", operand);
    let value = self.memory.get_post_indexed_data(operand, self.y_register);
    self.adc(value);
  }

  pub fn sta_zero_page(&mut self, index: u8) {
    trace!("STA storing 0x{:X} at 0x{:X}", self.accumulator, index);
    self.memory.set_zero_page(index, self.accumulator);
  }

  pub fn sta_zero_page_x(&mut self, operand: u8) {
    let index = operand.wrapping_add(self.x_register);
    trace!("STA storing 0x{:X} at 0x{:X}", self.accumulator, index);
    self.memory.set_zero_page(index, self.accumulator);
  }

  pub fn sta_absolute(&mut self, index: u16) {
    trace!("STA storing 0x{:X} at 0x{:X}", self.accumulator, index);
    self.memory.set(index, self.accumulator);
  }

  pub fn sta_absolute_x(&mut self, index: u16) {
    let index = index + self.x_register as u16;
    trace!("STA storing 0x{:X} at 0x{:X}", self.accumulator, index);
    self.memory.set(index, self.accumulator);
  }

  pub fn sta_absolute_y(&mut self, index: u16) {
    let index = index + self.y_register as u16;
    trace!("STA storing 0x{:X} at 0x{:X}", self.accumulator, index);
    self.memory.set(index, self.accumulator);
  }

  pub fn sta_indexed_x(&mut self, operand: u8) {
    let index = self.memory.get_pre_adjusted_index(operand, self.x_register);
    trace!("STA storing 0x{:X} at 0x{:X}", self.accumulator, index);
    self.memory.set(index, self.accumulator);
  }

  pub fn sta_indexed_y(&mut self, operand: u8) {
    let index = self.memory.get_pre_adjusted_index(operand, self.y_register);
    trace!("STA storing 0x{:X} at 0x{:X}", self.accumulator, index);
    self.memory.set(index, self.accumulator);
  }

  pub fn clc(&mut self) {
    trace!("CLC called");
    self.status_register.clear_carry_bit();
  }

  pub fn sec(&mut self) {
    trace!("SEC called");
    self.status_register.set_carry_bit();
  }

  pub fn cld(&mut self) {
    trace!("CLD called");
    self.status_register.clear_decimal_bit();
  }

  pub fn sed(&mut self) {
    trace!("SED called");
    self.status_register.set_decimal_bit();
  }

  pub fn cli(&mut self) {
    trace!("CLI called");
    self.status_register.clear_interrupt_bit();
  }

  pub fn sei(&mut self) {
    trace!("SEI called");
    self.status_register.set_interrupt_bit();
  }

  pub fn clv(&mut self) {
    trace!("CLV called");
    self.status_register.clear_overflow_bit();
  }

  /// Loads the accumulator with the value given
  ///
  /// Affects flags N Z
  pub fn lda(&mut self, value: u8) {
    let message = "LDA";
    trace!("{} called with value: 0x{:X}", message, value);
    self.accumulator = value;
    self.status_register.handle_n_flag(value, message);
    self.status_register.handle_z_flag(value, message);
  }
}

impl Display for CPU {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(
      f,
      "program_counter: 0x{:X}\nstack_pointer: 0x{:X}\naccumulator: 0x{:X}\nstatus_register: {}\nx_register: 0x{:X}\ny_register: 0x{:X}\n",
      self.program_counter, self.stack_pointer, self.accumulator, self.status_register, self.x_register, self.y_register
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn create_new_cpu() {
    let cpu = CPU::new();
    assert_eq!(cpu.accumulator, 0);
    assert_eq!(cpu.program_counter, 0);
    assert_eq!(cpu.stack_pointer, 0);
    assert_eq!(cpu.x_register, 0);
    assert_eq!(cpu.y_register, 0);
  }

  #[test]
  fn reset_cpu() {
    let mut cpu = CPU::new();
    cpu.stack_pointer = 23;
    cpu.program_counter = 23;
    cpu.accumulator = 23;
    cpu.x_register = 23;
    cpu.y_register = 23;
    cpu.status_register.set_break_bit();
    cpu.reset();
    assert_eq!(cpu.accumulator, 0);
    assert_eq!(cpu.program_counter, 0);
    assert_eq!(cpu.stack_pointer, 0);
    assert_eq!(cpu.x_register, 0);
    assert_eq!(cpu.y_register, 0);
  }

  #[test]
  fn adc_basic() {
    let mut cpu = CPU::new();
    cpu.accumulator = 0xAA;
    cpu.adc(0x11);
    assert_eq!(cpu.accumulator, 0xAA + 0x11);
  }

  #[test]
  fn adc_with_carry() {
    let mut cpu = CPU::new();
    cpu.accumulator = 0x00;
    cpu.status_register.set_carry_bit();
    cpu.adc(0x11);
    assert_eq!(cpu.accumulator, 0x12);
  }

  #[test]
  fn adc_zero_page() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x12, 10);
    cpu.accumulator = 0x32;
    cpu.adc_zero_page(0x12);
    assert_eq!(cpu.accumulator, 0x32 + 10);
  }

  #[test]
  fn adc_zero_page_indexed_no_wrap() {
    let mut cpu = CPU::new();
    cpu.accumulator = 0x32;
    cpu.x_register = 0x11;
    cpu.memory.set(0x23, 48);
    cpu.adc_zero_page_indexed(0x12);
    assert_eq!(cpu.accumulator, 0x32 + 48);
  }

  #[test]
  fn adc_zero_page_indexed_wrap() {
    let mut cpu = CPU::new();
    cpu.accumulator = 0x32;
    cpu.x_register = 0x11;
    cpu.memory.set(0x10, 48);
    cpu.adc_zero_page_indexed(0xFF);
    assert_eq!(cpu.accumulator, 0x32 + 48);
  }

  #[test]
  fn adc_absolute() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x1234, 0x56);
    cpu.accumulator = 0x10;
    cpu.adc_absolute(0x1234);
    assert_eq!(cpu.accumulator, 0x56 + 0x10);
  }

  #[test]
  fn adc_absolute_x() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x1234 + 0x56, 0x56);
    cpu.x_register = 0x56;
    cpu.accumulator = 0x10;
    cpu.adc_absolute_x(0x1234);
    assert_eq!(cpu.accumulator, 0x56 + 0x10);
  }

  #[test]
  fn adc_absolute_y() {
    let mut cpu = CPU::new();
    cpu.memory.set(0x5678 + 0x12, 0x90);
    cpu.y_register = 0x12;
    cpu.accumulator = 0xA0;
    cpu.adc_absolute_y(0x5678);
    assert_eq!(cpu.accumulator, 0x30);
  }

  #[test]
  fn sta_zero_page() {
    let mut cpu = CPU::new();
    cpu.accumulator = 0x34;
    cpu.sta_zero_page(0x55);
    assert_eq!(cpu.memory.get_zero_page(0x55), 0x34);
  }

  #[test]
  fn sta_zero_page_x() {
    let mut cpu = CPU::new();
    cpu.accumulator = 0x99;
    cpu.x_register = 0x34;
    cpu.sta_zero_page_x(0x55);
    assert_eq!(cpu.memory.get_zero_page(0x55 + 0x34), 0x99);
  }

  #[test]
  fn sta_absolute() {
    let mut cpu = CPU::new();
    cpu.accumulator = 0x99;
    cpu.sta_absolute(0x1255);
    assert_eq!(cpu.memory.get_u16(0x1255), 0x99);
  }

  #[test]
  fn sta_absolute_x() {
    let mut cpu = CPU::new();
    cpu.accumulator = 0x99;
    cpu.x_register = 0x10;
    cpu.sta_absolute_x(0x1255);
    assert_eq!(cpu.memory.get_u16(0x1265), 0x99);
  }

  #[test]
  fn sta_absolute_y() {
    let mut cpu = CPU::new();
    cpu.accumulator = 0x99;
    cpu.y_register = 0x20;
    cpu.sta_absolute_y(0x1275);
    assert_eq!(cpu.memory.get_u16(0x1295), 0x99);
  }

  #[test]
  fn sta_indexed_x() {
    let mut cpu = CPU::new();
    cpu.accumulator = 0x99;
    cpu.x_register = 0x20;
    cpu.memory.set(0x32, 0x20);
    cpu.memory.set(0x33, 0x25);
    cpu.sta_indexed_x(0x12);
    assert_eq!(cpu.memory.get_u16(0x2520), 0x99);
  }

  #[test]
  fn sta_indexed_y() {
    let mut cpu = CPU::new();
    cpu.accumulator = 0x45;
    cpu.y_register = 0x20;
    cpu.memory.set(0x76, 0x57);
    cpu.memory.set(0x77, 0xCB);
    cpu.sta_indexed_y(0x12);
    assert_eq!(cpu.memory.get_u16(0xCB69), 0x45);
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
}
