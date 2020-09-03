mod registers;
use registers::status_register::StatusRegister;

pub struct CPU {
  program_counter: u16,
  stack_pointer: u8,
  accumulator: u8,
  x_register: u8,
  y_register: u8,
  status_register: StatusRegister,
  memory: [u8; 0xFFFF],
}

impl CPU {
  pub fn new() -> CPU {
    CPU {
      program_counter: 0,
      stack_pointer: 0,
      accumulator: 0,
      x_register: 0,
      y_register: 0,
      status_register: StatusRegister::new(),
      memory: [0; 0xFFFF],
    }
  }

  pub fn reset(&mut self) {
    self.program_counter = 0;
    self.stack_pointer = 0;
    self.accumulator = 0;
    self.x_register = 0;
    self.y_register = 0;
    self.status_register.reset();
    self.memory = [0; 0xFFFF];
  }

  pub fn adc(&mut self, value: u8) {
    let (result, carry) = self.accumulator.overflowing_add(value);
    self.accumulator = result;
    if carry {
      self.status_register.set_carry_bit();
    }
    if result == 0 {
      self.status_register.set_zero_bit();
    }
  }

  pub fn adc_zero_page(&mut self, index: u8) {
    let value = self.memory[index as usize];
    self.adc(value);
  }

  pub fn adc_zero_page_indexed(&mut self, operand: u8) {
    let index = operand.wrapping_add(self.x_register);
    self.adc_zero_page(index);
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
    assert_eq!(cpu.status_register.get_register(), 0);
    assert_eq!(cpu.x_register, 0);
    assert_eq!(cpu.y_register, 0);
    for i in cpu.memory.iter() {
      assert_eq!(*i, 0);
    }
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
    cpu.memory = [12; 0xFFFF];
    cpu.reset();
    assert_eq!(cpu.accumulator, 0);
    assert_eq!(cpu.program_counter, 0);
    assert_eq!(cpu.stack_pointer, 0);
    assert_eq!(cpu.status_register.get_register(), 0);
    assert_eq!(cpu.x_register, 0);
    assert_eq!(cpu.y_register, 0);
    for i in cpu.memory.iter() {
      assert_eq!(*i, 0);
    }
  }

  #[test]
  fn adc_basic() {
    let mut cpu = CPU::new();
    cpu.accumulator = 0xAA;
    cpu.adc(0x11);
    assert_eq!(cpu.accumulator, 0xBB);
    assert_eq!(cpu.status_register.is_carry_bit_set(), false);
    assert_eq!(cpu.status_register.is_zero_bit_set(), false);
  }

  #[test]
  fn adc_carry_bit() {
    let mut cpu = CPU::new();
    cpu.accumulator = 0xFF;
    cpu.adc(0x11);
    assert_eq!(cpu.accumulator, 0x10);
    assert_eq!(cpu.status_register.is_carry_bit_set(), true);
    assert_eq!(cpu.status_register.is_zero_bit_set(), false);
  }

  #[test]
  fn adc_zero_bit() {
    let mut cpu = CPU::new();
    cpu.accumulator = 0xFF;
    cpu.adc(0x01);
    assert_eq!(cpu.accumulator, 0x00);
    assert_eq!(cpu.status_register.is_carry_bit_set(), true);
    assert_eq!(cpu.status_register.is_zero_bit_set(), true);
  }

  #[test]
  fn adc_zero_page() {
    let mut cpu = CPU::new();
    cpu.memory[0x12] = 10;
    cpu.accumulator = 0x32;
    cpu.adc_zero_page(0x12);
    assert_eq!(cpu.accumulator, 0x32 + 10);
    assert_eq!(cpu.status_register.is_carry_bit_set(), false);
    assert_eq!(cpu.status_register.is_zero_bit_set(), false);
  }

  #[test]
  fn adc_zero_page_indexed_no_wrap() {
    let mut cpu = CPU::new();
    cpu.accumulator = 0x32;
    cpu.x_register = 0x11;
    cpu.memory[0x23] = 48;
    cpu.adc_zero_page_indexed(0x12);
    assert_eq!(cpu.accumulator, 0x32 + 48);
    assert_eq!(cpu.status_register.is_carry_bit_set(), false);
    assert_eq!(cpu.status_register.is_zero_bit_set(), false);
  }

  #[test]
  fn adc_zero_page_indexed_wrap() {
    let mut cpu = CPU::new();
    cpu.accumulator = 0x32;
    cpu.x_register = 0x11;
    cpu.memory[0x10] = 48;
    cpu.adc_zero_page_indexed(0xFF);
    assert_eq!(cpu.accumulator, 0x32 + 48);
    assert_eq!(cpu.status_register.is_carry_bit_set(), false);
    assert_eq!(cpu.status_register.is_zero_bit_set(), false);
  }
}
