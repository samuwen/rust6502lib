mod memory;
mod registers;

use log::trace;
use memory::Memory;
use registers::{GeneralRegister, ProgramCounter, StackPointer, StatusRegister};
use std::fmt::{Display, Formatter, Result};
use std::time::Duration;

const STARTING_MEMORY_BLOCK: u16 = 0x8000;
const TIME_PER_CYCLE: u64 = 1790;

// TODO: Memory space is 256 pages of 256 bytes. If a page index (hi byte) is incremented
// Cycles should be incremented as well per "page boundary crossing"

/// An emulated CPU for the 6502 processor.
pub struct CPU {
  program_counter: ProgramCounter,
  stack_pointer: StackPointer,
  accumulator: GeneralRegister,
  x_register: GeneralRegister,
  y_register: GeneralRegister,
  status_register: StatusRegister,
  memory: Memory,
  cycle_counter: u8,
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
      cycle_counter: 0,
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
    self.cycle_counter = 0;
    trace!("CPU Reset")
  }

  fn load_program_into_memory(&mut self, program: &Vec<u8>) {
    let mut memory_address = STARTING_MEMORY_BLOCK;
    for byte in program.iter() {
      self.memory.set(memory_address, *byte);
      memory_address += 1;
    }
  }

  /// Runs a program while there are opcodes to handle. This will change when we actually have
  /// a real data set to operate against.
  pub fn run(&mut self, program: Vec<u8>) {
    self.load_program_into_memory(&program);
    loop {
      let opcode = self.program_counter.get_single_operand(&self.memory);
      match opcode {
        0x24 => self.zero_page("BIT", &mut CPU::bit),
        0x29 => self.immediate("AND", &mut CPU::and),
        0x2C => self.absolute("BIT", &mut CPU::bit),
        0x61 => self.indexed_x("ADC", &mut CPU::adc),
        0x65 => self.zero_page("ADC", &mut CPU::adc),
        0x69 => self.immediate("ADC", &mut CPU::adc),
        0x6D => self.absolute("ADC", &mut CPU::adc),
        0x71 => self.indexed_y("ADC", &mut CPU::adc),
        0x75 => self.zp_reg("ADC", self.x_register.get(), &mut CPU::adc),
        0x79 => self.absolute_x("ADC", &mut CPU::adc),
        0x7D => self.absolute_y("ADC", &mut CPU::adc),
        _ => (),
      }
      std::thread::sleep(Duration::from_micros(
        self.cycle_counter as u64 * TIME_PER_CYCLE,
      ));
    }
  }

  /*
  ============================================================================================
                                  Generic operations
  ============================================================================================
  */

  fn immediate<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let op = self.program_counter.get_single_operand(&self.memory);
    trace!("{} immediate called with operand:0x{:X}", name, op);
    cb(self, op);
  }

  /// Generically handles zero page retrieval operations and calls a callback when complete
  fn zero_page<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let index = self.program_counter.get_single_operand(&self.memory);
    trace!("{} zero page called with index: 0x{:X}", name, index);
    let value = self.memory.get_zero_page(index);
    cb(self, value);
  }

  fn zp_reg<F: FnMut(&mut Self, u8)>(&mut self, name: &str, reg_val: u8, cb: &mut F) {
    let op = self.program_counter.get_single_operand(&self.memory);
    trace!("{} zero page x called with operand: 0x{:X}", name, op);
    let index = op.wrapping_add(reg_val);
    cb(self, self.memory.get_zero_page(index));
  }

  fn absolute<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let ops = self.program_counter.get_two_operands(&self.memory);
    let index = u16::from_le_bytes(ops);
    trace!("{} absolute called with index: 0x{:X}", name, index);
    let value = self.memory.get_u16(index);
    cb(self, value);
  }

  fn absolute_x<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let ops = self.program_counter.get_two_operands(&self.memory);
    let index = u16::from_le_bytes(ops);
    trace!("{} absolute x called with index: 0x{:X}", name, index);
    let value = self
      .memory
      .get_u16_and_register(index, self.x_register.get());
    cb(self, value);
  }

  fn absolute_y<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let ops = self.program_counter.get_two_operands(&self.memory);
    let index = u16::from_le_bytes(ops);
    trace!("{} absolute y called with index: 0x{:X}", name, index);
    let value = self
      .memory
      .get_u16_and_register(index, self.y_register.get());
    cb(self, value);
  }

  /// AKA Indexed indirect AKA pre-indexed
  fn indexed_x<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let op = self.program_counter.get_single_operand(&self.memory);
    trace!("{} indexed x called with operand: 0x{:X}", name, op);
    let value = self.memory.get_pre_indexed_data(op, self.x_register.get());
    cb(self, value);
  }

  /// AKA Indirect indexed AKA post-indexed
  fn indexed_y<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let op = self.program_counter.get_single_operand(&self.memory);
    trace!("{} indexed y called with operand: 0x{:X}", name, op);
    let value = self.memory.get_post_indexed_data(op, self.y_register.get());
    cb(self, value);
  }

  fn flag_operation<F: FnMut(&mut StatusRegister)>(&mut self, name: &str, cb: &mut F) {
    trace!("{} called", name);
    cb(&mut self.status_register);
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
    let (result, carry) = self
      .accumulator
      .get()
      .overflowing_add(value.wrapping_add(modifier));
    self.accumulator.set(result);
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_v_flag(result, message, carry);
    self.status_register.handle_z_flag(result, message);
    self.status_register.handle_c_flag(message, carry);
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

  pub fn asl_zero_page(&mut self) {
    let index = self.program_counter.get_single_operand(&self.memory);
    trace!("ASL zero page called with index: 0x{:X}", index);
    let value = self.memory.get_zero_page(index);
    let result = self.asl(value);
    self.memory.set_zero_page(index, result);
  }

  pub fn asl_zero_page_x(&mut self) {
    let index = self.program_counter.get_single_operand(&self.memory);
    trace!("ASL zero page x called with index: 0x{:X}", index);
    let mod_index = index.wrapping_add(self.x_register.get());
    let value = self.memory.get_zero_page(mod_index);
    let result = self.asl(value);
    self.memory.set_zero_page(mod_index, result);
  }

  pub fn asl_absolute(&mut self) {
    let ops = self.program_counter.get_two_operands(&self.memory);
    let index = u16::from_le_bytes(ops);
    trace!("ASL absolute called with index: 0x{:X}", index);
    let value = self.memory.get_u16(index);
    let result = self.asl(value);
    self.memory.set(index, result);
  }

  pub fn asl_absolute_x(&mut self) {
    let ops = self.program_counter.get_two_operands(&self.memory);
    let index = u16::from_le_bytes(ops);
    trace!("ASL absolute called with index: 0x{:X}", index);
    let mod_index = index.wrapping_add(self.x_register.get() as u16);
    let value = self.memory.get_u16(mod_index);
    let result = self.asl(value);
    self.memory.set(mod_index, result);
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

  fn branch(&mut self, condition: bool, op: u8) {
    if condition {
      if op > 0x7F {
        println!("decreasing");
        self.program_counter.decrease(!op + 1);
      } else {
        self.program_counter.increase(op);
      }
    }
  }

  pub fn bpl(&mut self) {
    let op = self.program_counter.get_single_operand(&self.memory);
    self.branch(!self.status_register.is_negative_bit_set(), op);
  }

  pub fn bmi(&mut self) {
    let op = self.program_counter.get_single_operand(&self.memory);
    self.branch(self.status_register.is_negative_bit_set(), op);
  }

  pub fn bvc(&mut self) {
    let op = self.program_counter.get_single_operand(&self.memory);
    self.branch(!self.status_register.is_overflow_bit_set(), op);
  }

  pub fn bvs(&mut self) {
    let op = self.program_counter.get_single_operand(&self.memory);
    self.branch(self.status_register.is_overflow_bit_set(), op);
  }

  pub fn bcc(&mut self) {
    let op = self.program_counter.get_single_operand(&self.memory);
    self.branch(!self.status_register.is_carry_bit_set(), op);
  }

  pub fn bcs(&mut self) {
    let op = self.program_counter.get_single_operand(&self.memory);
    self.branch(self.status_register.is_carry_bit_set(), op);
  }

  pub fn bne(&mut self) {
    let op = self.program_counter.get_single_operand(&self.memory);
    self.branch(!self.status_register.is_zero_bit_set(), op);
  }

  pub fn beq(&mut self) {
    let op = self.program_counter.get_single_operand(&self.memory);
    self.branch(self.status_register.is_zero_bit_set(), op);
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

  pub fn sta_zero_page(&mut self) {
    let index = self.program_counter.get_single_operand(&self.memory);
    trace!(
      "STA storing 0x{:X} at 0x{:X}",
      self.accumulator.get(),
      index
    );
    self.memory.set_zero_page(index, self.accumulator.get());
  }

  pub fn sta_zero_page_x(&mut self) {
    let index = self.program_counter.get_single_operand(&self.memory);
    trace!(
      "STA storing 0x{:X} at 0x{:X}",
      self.accumulator.get(),
      index
    );
    self.memory.set_zero_page(
      index.wrapping_add(self.x_register.get()),
      self.accumulator.get(),
    );
  }

  pub fn sta_absolute(&mut self) {
    let ops = self.program_counter.get_two_operands(&self.memory);
    let index = u16::from_le_bytes(ops);
    trace!(
      "STA storing 0x{:X} at 0x{:X}",
      self.accumulator.get(),
      index
    );
    self.memory.set(index, self.accumulator.get());
  }

  pub fn sta_absolute_x(&mut self) {
    let ops = self.program_counter.get_two_operands(&self.memory);
    let index = u16::from_le_bytes(ops).wrapping_add(self.x_register.get() as u16);
    trace!(
      "STA storing 0x{:X} at 0x{:X}",
      self.accumulator.get(),
      index
    );
    self.memory.set(index, self.accumulator.get());
  }

  pub fn sta_absolute_y(&mut self) {
    let ops = self.program_counter.get_two_operands(&self.memory);
    let index = u16::from_le_bytes(ops).wrapping_add(self.y_register.get() as u16);
    trace!(
      "STA storing 0x{:X} at 0x{:X}",
      self.accumulator.get(),
      index
    );
    self.memory.set(index, self.accumulator.get());
  }

  pub fn sta_indexed_x(&mut self) {
    let operand = self.program_counter.get_single_operand(&self.memory);
    let index = self
      .memory
      .get_pre_adjusted_index(operand, self.x_register.get());
    trace!(
      "STA storing 0x{:X} at 0x{:X}",
      self.accumulator.get(),
      index
    );
    self.memory.set(index, self.accumulator.get());
  }

  pub fn sta_indexed_y(&mut self) {
    let operand = self.program_counter.get_single_operand(&self.memory);
    let index = self
      .memory
      .get_post_adjusted_index(operand, self.y_register.get());
    trace!(
      "STA storing 0x{:X} at 0x{:X}",
      self.accumulator.get(),
      index
    );
    self.memory.set(index, self.accumulator.get());
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
  use rand::distributions::uniform::SampleUniform;
  use rand::prelude::*;
  use rand::{thread_rng, Rng};
  use test_case::test_case;

  fn rand_range<T: SampleUniform>(min: T, max: T) -> T {
    let mut rng = thread_rng();
    rng.gen_range(min, max)
  }

  fn no_wrap() -> u8 {
    rand_range(0x5, 0x7F)
  }

  fn wrap() -> u8 {
    rand_range(0x7F, 0xFF)
  }

  fn r_lsb() -> u8 {
    rand_range(0x5, 0xFE)
  }

  fn r_u16() -> u16 {
    rand_range(0x5, 0xFFFE)
  }

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
    cpu.program_counter.increase(1);
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

  fn setup(acc: u8) -> CPU {
    let mut cpu = CPU::new();
    cpu.accumulator.set(acc);
    cpu
  }

  fn setup_zp(acc: u8, index: u8, value: u8) -> CPU {
    let mut cpu = setup(acc);
    cpu.memory.set_zero_page(index, value);
    cpu.memory.set_zero_page(1, index);
    cpu.program_counter.increase(1);
    cpu
  }

  fn setup_zp_reg(acc: u8, index: u8, reg: u8, value: u8) -> CPU {
    let mut cpu = setup(acc);
    cpu.memory.set_zero_page(index.wrapping_add(reg), value);
    cpu.memory.set_zero_page(1, index);
    cpu.program_counter.increase(1);
    cpu
  }

  fn abs_set(ops: [u8; 2]) -> CPU {
    let mut cpu = CPU::new();
    cpu.memory.set_zero_page(1, ops[0]);
    cpu.memory.set_zero_page(2, ops[1]);
    cpu.program_counter.increase(1);
    cpu
  }

  fn setup_abs(index: u16, value: u8) -> CPU {
    let mut cpu = abs_set(index.to_le_bytes());
    cpu.memory.set(index, value);
    cpu
  }

  fn setup_abs_reg(index: u16, reg: u8, value: u8) -> CPU {
    let mut cpu = abs_set(index.to_le_bytes());
    cpu.memory.set(index.wrapping_add(reg as u16), value);
    cpu
  }

  fn setup_indexed_x(acc: u8, value: u8, v1: u8, v2: u8, op: u8, reg_v: u8) -> CPU {
    let mut cpu = setup(acc);
    let index = u16::from_le_bytes([v1, v2]);
    cpu.memory.set(index, value);
    let start = op.wrapping_add(reg_v);
    cpu.memory.set_zero_page(start, v1);
    cpu.memory.set_zero_page(start.wrapping_add(1), v2);
    cpu.memory.set_zero_page(1, op);
    cpu.program_counter.increase(1);
    cpu
  }

  fn setup_indexed_y(acc: u8, value: u8, v1: u8, v2: u8, op: u8, reg_v: u8) -> CPU {
    let mut cpu = setup(acc);
    let index = u16::from_le_bytes([v1, v2]);
    cpu.memory.set(index + reg_v as u16, value);
    cpu.memory.set_zero_page(op, v1);
    cpu.memory.set_zero_page(op.wrapping_add(1), v2);
    cpu.memory.set(1, op);
    cpu.program_counter.increase(1);
    cpu
  }

  fn setup_carry(cpu: &mut CPU, carry: u8) {
    if carry > 0 {
      cpu.status_register.set_carry_bit();
    }
  }

  fn setup_branch(val: u8, pc_start: u8) -> CPU {
    let mut cpu = CPU::new();
    cpu.program_counter.increase(pc_start);
    cpu.memory.set_zero_page(pc_start, val);
    cpu
  }

  #[test_case(no_wrap(), no_wrap(), 0; "adc without wrap without carry set")]
  #[test_case(wrap(), wrap(), 0; "adc with wrap without carry set")]
  #[test_case(no_wrap(), no_wrap(), 1; "adc without wrap with carry set")]
  #[test_case(wrap(), wrap(), 1; "adc with wrap with carry set")]
  fn adc(acc: u8, operand: u8, carry: u8) {
    let mut cpu = setup(acc);
    setup_carry(&mut cpu, carry);
    cpu.adc(operand);
    assert_eq!(
      cpu.accumulator.get(),
      acc.wrapping_add(operand).wrapping_add(carry)
    );
    cpu.program_counter.increase(1);
    assert_eq!(cpu.program_counter.get(), 1);
  }

  #[test_case(no_wrap(), r_lsb(), no_wrap(), 0; "zero page without wrap without carry")]
  #[test_case(wrap(), r_lsb(), wrap(), 0; "zero page with wrap without carry")]
  #[test_case(no_wrap(), r_lsb(), no_wrap(), 1; "zero page without wrap with carry")]
  #[test_case(wrap(), r_lsb(), wrap(), 1; "zero page with wrap with carry")]
  fn adc_zero_page(acc: u8, index: u8, value: u8, carry: u8) {
    let mut cpu = setup_zp(acc, index, value);
    setup_carry(&mut cpu, carry);
    cpu.zero_page("ADC", &mut CPU::adc);
    assert_eq!(
      cpu.accumulator.get(),
      acc.wrapping_add(value).wrapping_add(carry)
    );
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(no_wrap(), r_lsb(), random(), no_wrap(), 0; "indexed zero page without wrap without carry")]
  #[test_case(wrap(), r_lsb(), random(), wrap(), 0; "indexed zero page with wrap without carry")]
  #[test_case(no_wrap(), r_lsb(), random(), no_wrap(), 1; "indexed zero page without wrap with carry")]
  #[test_case(wrap(), r_lsb(), random(), wrap(), 1; "indexed zero page with wrap with carry")]
  fn adc_zero_page_x(acc: u8, index: u8, x: u8, value: u8, carry: u8) {
    let mut cpu = setup_zp_reg(acc, index, x, value);
    setup_carry(&mut cpu, carry);
    cpu.x_register.set(x);
    cpu.zp_reg("ADC", x, &mut CPU::adc);
    assert_eq!(
      cpu.accumulator.get(),
      acc.wrapping_add(value).wrapping_add(carry)
    );
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(r_u16(), no_wrap(), no_wrap(), 0; "absolute without wrap without carry")]
  #[test_case(r_u16(), wrap(), wrap(), 0; "absolute with wrap without carry")]
  #[test_case(r_u16(), no_wrap(), no_wrap(), 1; "absolute without wrap with carry")]
  #[test_case(r_u16(), wrap(), wrap(), 1; "absolute with wrap with carry")]
  fn adc_absolute(index: u16, value: u8, acc: u8, carry: u8) {
    let mut cpu = setup_abs(index, value);
    cpu.accumulator.set(acc);
    setup_carry(&mut cpu, carry);
    cpu.absolute("ADC", &mut CPU::adc);
    assert_eq!(
      cpu.accumulator.get(),
      value.wrapping_add(acc).wrapping_add(carry)
    );
    assert_eq!(cpu.program_counter.get(), 3);
  }

  #[test_case(no_wrap(), r_u16(), random(), no_wrap(), 0; "absolute x without wrap without carry")]
  #[test_case(wrap(), r_u16(), random(), wrap(), 0; "absolute x with wrap without carry")]
  #[test_case(no_wrap(), r_u16(), random(), no_wrap(), 1; "absolute x without wrap with carry")]
  #[test_case(wrap(), r_u16(), random(), wrap(), 1; "absolute x with wrap with carry")]
  fn adc_absolute_x(acc: u8, index: u16, x: u8, value: u8, carry: u8) {
    let mut cpu = setup_abs_reg(index, x, value);
    cpu.accumulator.set(acc);
    setup_carry(&mut cpu, carry);
    cpu.x_register.set(x);
    cpu.absolute_x("ADC", &mut CPU::adc);
    assert_eq!(cpu.accumulator.get(), value.wrapping_add(acc) + carry);
    assert_eq!(cpu.program_counter.get(), 3);
  }

  #[test_case(no_wrap(), r_u16(), random(), no_wrap(), 0; "absolute y without wrap without carry")]
  #[test_case(wrap(), r_u16(), random(), wrap(), 0; "absolute y with wrap without carry")]
  #[test_case(no_wrap(), r_u16(), random(), no_wrap(), 1; "absolute y without wrap with carry")]
  #[test_case(wrap(), r_u16(), random(), wrap(), 1; "absolute y with wrap with carry")]
  fn adc_absolute_y(acc: u8, index: u16, y: u8, value: u8, carry: u8) {
    let mut cpu = setup_abs_reg(index, y, value);
    cpu.accumulator.set(acc);
    setup_carry(&mut cpu, carry);
    cpu.y_register.set(y);
    cpu.absolute_y("ADC", &mut CPU::adc);
    assert_eq!(cpu.accumulator.get(), value.wrapping_add(acc) + carry);
    assert_eq!(cpu.program_counter.get(), 3);
  }

  #[test_case(no_wrap(), r_lsb(), random(), random(), random(), no_wrap(), 0; "indexed x without wrap without carry")]
  #[test_case(wrap(), r_lsb(), random(), random(), random(), wrap(), 0; "indexed x with wrap without carry")]
  #[test_case(no_wrap(), r_lsb(), random(), random(), random(), no_wrap(), 1; "indexed x without wrap with carry")]
  #[test_case(wrap(), r_lsb(), random(), random(), random(), wrap(), 1; "indexed x with wrap with carry")]
  fn adc_indexed_x(acc: u8, operand: u8, x: u8, v1: u8, v2: u8, value: u8, carry: u8) {
    let mut cpu = setup_indexed_x(acc, value, v1, v2, operand, x);
    if operand.wrapping_add(x) == 0 {
      cpu = setup_indexed_x(acc, value, v1, v2, operand, random());
    }
    setup_carry(&mut cpu, carry);
    cpu.x_register.set(x);
    cpu.indexed_x("ADC", &mut CPU::adc);
    assert_eq!(
      cpu.accumulator.get(),
      value.wrapping_add(acc).wrapping_add(carry)
    );
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(no_wrap(), r_lsb(), random(), 0x30, 0x40, no_wrap(), 0; "indexed y without wrap without carry")]
  #[test_case(wrap(), r_lsb(), random(), 0x30, 0x40, wrap(), 0; "indexed y with wrap without carry")]
  #[test_case(no_wrap(), r_lsb(), random(), 0x30, 0x40, no_wrap(), 1; "indexed y without wrap with carry")]
  #[test_case(wrap(), r_lsb(), random(), 0x30, 0x40, wrap(), 1; "indexed y with wrap with carry")]
  fn adc_indexed_y(acc: u8, operand: u8, x: u8, v1: u8, v2: u8, value: u8, carry: u8) {
    let mut cpu = setup_indexed_y(acc, value, v1, v2, operand, x);
    setup_carry(&mut cpu, carry);
    cpu.y_register.set(x);
    cpu.indexed_y("ADC", &mut CPU::adc);
    assert_eq!(
      cpu.accumulator.get(),
      value.wrapping_add(acc).wrapping_add(carry)
    );
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(random(), random())]
  fn and_basic(acc: u8, operand: u8) {
    let mut cpu = setup(acc);
    cpu.and(operand);
    // not doing any PC logic just calling the function directly
    cpu.program_counter.increase(2);
    assert_eq!(cpu.accumulator.get(), acc & operand);
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(random(), random(), random())]
  fn and_zero_page(acc: u8, index: u8, value: u8) {
    let mut cpu = setup_zp(acc, index, value);
    cpu.zero_page("AND", &mut CPU::and);
    assert_eq!(cpu.accumulator.get(), acc & value);
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(random(), r_lsb(), random(), random())]
  fn and_zero_page_x(acc: u8, index: u8, value: u8, x: u8) {
    let mut cpu = setup_zp_reg(acc, index, x, value);
    cpu.x_register.set(x);
    cpu.zp_reg("AND", x, &mut CPU::and);
    assert_eq!(cpu.accumulator.get(), acc & value);
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(random(), r_u16(), random())]
  fn and_absolute(acc: u8, index: u16, value: u8) {
    let mut cpu = setup_abs(index, value);
    cpu.accumulator.set(acc);
    cpu.absolute("AND", &mut CPU::and);
    assert_eq!(cpu.accumulator.get(), acc & value);
    assert_eq!(cpu.program_counter.get(), 3);
  }

  #[test_case(random(), r_u16(), random(), random())]
  fn and_absolute_x(acc: u8, index: u16, value: u8, x: u8) {
    let mut cpu = setup_abs_reg(index, x, value);
    cpu.accumulator.set(acc);
    cpu.x_register.set(x);
    cpu.absolute_x("AND", &mut CPU::and);
    assert_eq!(cpu.accumulator.get(), acc & value);
    assert_eq!(cpu.program_counter.get(), 3);
  }

  #[test_case(random(), r_u16(), random(), random())]
  fn and_absolute_y(acc: u8, index: u16, value: u8, y: u8) {
    let mut cpu = setup_abs_reg(index, y, value);
    cpu.accumulator.set(acc);
    cpu.y_register.set(y);
    cpu.absolute_y("AND", &mut CPU::and);
    assert_eq!(cpu.accumulator.get(), acc & value);
    assert_eq!(cpu.program_counter.get(), 3);
  }

  #[test_case(random(), r_lsb(), random(), random(), random(), random())]
  fn and_indexed_x(acc: u8, operand: u8, x: u8, v1: u8, v2: u8, value: u8) {
    let mut cpu = setup_indexed_x(acc, value, v1, v2, operand, x);
    cpu.x_register.set(x);
    cpu.indexed_x("AND", &mut CPU::and);
    assert_eq!(cpu.accumulator.get(), value & acc);
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(random(), r_lsb(), random(), random(), random(), random())]
  fn and_indexed_y(acc: u8, operand: u8, y: u8, v1: u8, v2: u8, value: u8) {
    let mut cpu = setup_indexed_y(acc, value, v1, v2, operand, y);
    cpu.y_register.set(y);
    cpu.indexed_y("AND", &mut CPU::and);
    assert_eq!(cpu.accumulator.get(), value & acc);
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(no_wrap(); "without wrap")]
  #[test_case(wrap(); "with wrap")]
  fn asl(val: u8) {
    let mut cpu = CPU::new();
    let result = cpu.asl(val);
    assert_eq!(result, val.wrapping_shl(1));
  }

  #[test_case(no_wrap(); "without wrap")]
  #[test_case(wrap(); "with wrap")]
  fn asl_accumulator(acc: u8) {
    let mut cpu = setup(acc);
    cpu.asl_accumulator();
    cpu.program_counter.increase(1);
    assert_eq!(cpu.accumulator.get(), acc.wrapping_shl(1));
    assert_eq!(cpu.program_counter.get(), 1);
  }

  #[test_case(r_lsb(), no_wrap(); "without wrap")]
  #[test_case(r_lsb(), wrap(); "with wrap")]
  fn asl_zero_page(index: u8, value: u8) {
    let mut cpu = setup_zp(0, index, value);
    cpu.asl_zero_page();
    assert_eq!(cpu.memory.get_zero_page(index), value.wrapping_shl(1));
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(no_wrap(), no_wrap(), no_wrap(); "without shift wrap without index wrap")]
  #[test_case(no_wrap(), wrap(), no_wrap(); "with shift wrap without index wrap")]
  #[test_case(wrap(), no_wrap(), wrap(); "without shift wrap with index wrap")]
  #[test_case(wrap(), wrap(), wrap(); "with shift wrap with index wrap")]
  fn asl_zero_page_x(index: u8, value: u8, x: u8) {
    let mod_index = index.wrapping_add(x);
    let mut cpu = setup_zp_reg(0, index, x, value);
    cpu.x_register.set(x);
    cpu.asl_zero_page_x();
    assert_eq!(cpu.memory.get_zero_page(mod_index), value.wrapping_shl(1));
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(random(), no_wrap(); "without wrap")]
  #[test_case(random(), wrap(); "with wrap")]
  fn asl_absolute(index: u16, value: u8) {
    let mut cpu = setup_abs(index, value);
    cpu.asl_absolute();
    assert_eq!(cpu.memory.get_u16(index), value.wrapping_shl(1));
    assert_eq!(cpu.program_counter.get(), 3);
  }

  #[test_case(random(), no_wrap(), no_wrap(); "without wrap")]
  #[test_case(random(), no_wrap(), wrap(); "with wrap")]
  fn asl_absolute_x(index: u16, value: u8, x: u8) {
    let mod_index = index.wrapping_add(x as u16);
    let mut cpu = setup_abs_reg(index, x, value);
    cpu.x_register.set(x);
    cpu.asl_absolute_x();
    assert_eq!(cpu.memory.get_u16(mod_index), value.wrapping_shl(1));
    assert_eq!(cpu.program_counter.get(), 3);
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

  fn is_zero_bit_set(val: u8) -> bool {
    val == 0
  }

  fn is_overflow_bit_set(val: u8) -> bool {
    val & 0x40 > 0
  }

  fn is_negative_bit_set(val: u8) -> bool {
    val & 0x80 > 0
  }

  #[test_case(random())]
  fn bit_overflow_bit(val: u8) {
    let mut cpu = CPU::new();
    cpu.bit(val);
    assert_eq!(
      cpu.status_register.is_overflow_bit_set(),
      is_overflow_bit_set(val)
    );
  }

  #[test_case(random())]
  fn bit_negative_bit(val: u8) {
    let mut cpu = CPU::new();
    cpu.bit(val);
    assert_eq!(
      cpu.status_register.is_negative_bit_set(),
      is_negative_bit_set(val)
    );
  }

  #[test_case(random(), r_lsb(), random())]
  fn bit_zero_page(val: u8, index: u8, acc: u8) {
    let mut cpu = setup_zp(acc, index, val);
    cpu.zero_page("BIT", &mut CPU::bit);
    assert_eq!(
      cpu.status_register.is_zero_bit_set(),
      is_zero_bit_set(val & acc)
    );
    assert_eq!(
      cpu.status_register.is_negative_bit_set(),
      is_negative_bit_set(val)
    );
    assert_eq!(
      cpu.status_register.is_overflow_bit_set(),
      is_overflow_bit_set(val)
    );
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(r_u16(), random(), random())]
  fn bit_absolute(index: u16, val: u8, acc: u8) {
    let mut cpu = setup_abs(index, val);
    cpu.accumulator.set(acc);
    cpu.absolute("BIT", &mut CPU::bit);
    assert_eq!(
      cpu.status_register.is_zero_bit_set(),
      is_zero_bit_set(val & acc)
    );
    assert_eq!(
      cpu.status_register.is_negative_bit_set(),
      is_negative_bit_set(val)
    );
    assert_eq!(
      cpu.status_register.is_overflow_bit_set(),
      is_overflow_bit_set(val)
    );
    assert_eq!(cpu.program_counter.get(), 3);
  }

  fn get_branch_result(val: u8, pc_start: u8) -> u8 {
    match val > 0x7F {
      true => pc_start - (!val + 1),
      false => pc_start + val,
    }
  }

  #[test_case(wrap(), wrap(); "Branch PC goes backwards")]
  #[test_case(no_wrap(), no_wrap(); "Branch PC goes forwards")]
  fn branch(val: u8, pc_start: u8) {
    let mut cpu = setup_branch(val, pc_start);
    cpu.branch(true, val);
    let result = get_branch_result(val, pc_start);
    assert_eq!(cpu.program_counter.get(), result as usize);
  }

  #[test_case(random(), random(); "No branch PC doesnt do anything")]
  fn no_branch(val: u8, pc_start: u8) {
    let mut cpu = setup_branch(val, pc_start);
    cpu.branch(false, val);
    assert_eq!(cpu.program_counter.get(), pc_start as usize);
  }

  #[test_case(true, no_wrap(), no_wrap())]
  #[test_case(false, no_wrap(), no_wrap())]
  fn bpl(set_bit: bool, val: u8, pc_start: u8) {
    let mut cpu = setup_branch(val, pc_start);
    let result;
    if set_bit {
      cpu.status_register.set_negative_bit();
      result = pc_start;
    } else {
      result = get_branch_result(val, pc_start);
    }
    cpu.bpl();
    // increment by 1 for the opcode
    assert_eq!(cpu.program_counter.get(), result as usize + 1);
  }

  #[test_case(true, no_wrap(), no_wrap())]
  #[test_case(false, no_wrap(), no_wrap())]
  fn bmi(set_bit: bool, val: u8, pc_start: u8) {
    let mut cpu = setup_branch(val, pc_start);
    let result;
    if set_bit {
      cpu.status_register.set_negative_bit();
      result = get_branch_result(val, pc_start);
    } else {
      result = pc_start;
    }
    cpu.bmi();
    // increment by 1 for the opcode
    assert_eq!(cpu.program_counter.get(), result as usize + 1);
  }

  #[test_case(true, no_wrap(), no_wrap())]
  #[test_case(false, no_wrap(), no_wrap())]
  fn bvc(set_bit: bool, val: u8, pc_start: u8) {
    let mut cpu = setup_branch(val, pc_start);
    let result;
    if set_bit {
      cpu.status_register.set_overflow_bit();
      result = pc_start;
    } else {
      result = get_branch_result(val, pc_start);
    }
    cpu.bvc();
    // increment by 1 for the opcode
    assert_eq!(cpu.program_counter.get(), result as usize + 1);
  }

  #[test_case(true, no_wrap(), no_wrap())]
  #[test_case(false, no_wrap(), no_wrap())]
  fn bvs(set_bit: bool, val: u8, pc_start: u8) {
    let mut cpu = setup_branch(val, pc_start);
    let result;
    if set_bit {
      cpu.status_register.set_overflow_bit();
      result = get_branch_result(val, pc_start);
    } else {
      result = pc_start;
    }
    cpu.bvs();
    // increment by 1 for the opcode
    assert_eq!(cpu.program_counter.get(), result as usize + 1);
  }

  #[test_case(true, no_wrap(), no_wrap())]
  #[test_case(false, no_wrap(), no_wrap())]
  fn bcc(set_bit: bool, val: u8, pc_start: u8) {
    let mut cpu = setup_branch(val, pc_start);
    let result;
    if set_bit {
      cpu.status_register.set_carry_bit();
      result = pc_start;
    } else {
      result = get_branch_result(val, pc_start);
    }
    cpu.bcc();
    // increment by 1 for the opcode
    assert_eq!(cpu.program_counter.get(), result as usize + 1);
  }

  #[test_case(true, no_wrap(), no_wrap())]
  #[test_case(false, no_wrap(), no_wrap())]
  fn bcs(set_bit: bool, val: u8, pc_start: u8) {
    let mut cpu = setup_branch(val, pc_start);
    let result;
    if set_bit {
      cpu.status_register.set_carry_bit();
      result = get_branch_result(val, pc_start);
    } else {
      result = pc_start;
    }
    cpu.bcs();
    // increment by 1 for the opcode
    assert_eq!(cpu.program_counter.get(), result as usize + 1);
  }

  #[test_case(true, no_wrap(), no_wrap())]
  #[test_case(false, no_wrap(), no_wrap())]
  fn bne(set_bit: bool, val: u8, pc_start: u8) {
    let mut cpu = setup_branch(val, pc_start);
    let result;
    if set_bit {
      cpu.status_register.set_zero_bit();
      result = pc_start;
    } else {
      result = get_branch_result(val, pc_start);
    }
    cpu.bne();
    // increment by 1 for the opcode
    assert_eq!(cpu.program_counter.get(), result as usize + 1);
  }

  #[test_case(true, no_wrap(), no_wrap())]
  #[test_case(false, no_wrap(), no_wrap())]
  fn beq(set_bit: bool, val: u8, pc_start: u8) {
    let mut cpu = setup_branch(val, pc_start);
    let result;
    if set_bit {
      cpu.status_register.set_zero_bit();
      result = get_branch_result(val, pc_start);
    } else {
      result = pc_start;
    }
    cpu.beq();
    // increment by 1 for the opcode
    assert_eq!(cpu.program_counter.get(), result as usize + 1);
  }

  #[test]
  fn clc() {
    let mut cpu = CPU::new();
    cpu.status_register.set_carry_bit();
    cpu.clc();
    cpu.program_counter.increase(1);
    assert_eq!(cpu.status_register.is_carry_bit_set(), false);
    assert_eq!(cpu.program_counter.get(), 1);
  }

  #[test]
  fn cld() {
    let mut cpu = CPU::new();
    cpu.status_register.set_decimal_bit();
    cpu.cld();
    cpu.program_counter.increase(1);
    assert_eq!(cpu.status_register.is_decimal_bit_set(), false);
    assert_eq!(cpu.program_counter.get(), 1);
  }

  #[test]
  fn cli() {
    let mut cpu = CPU::new();
    cpu.status_register.set_interrupt_bit();
    cpu.cli();
    cpu.program_counter.increase(1);
    assert_eq!(cpu.status_register.is_interrupt_bit_set(), false);
    assert_eq!(cpu.program_counter.get(), 1);
  }

  #[test]
  fn clv() {
    let mut cpu = CPU::new();
    cpu.status_register.set_overflow_bit();
    cpu.clv();
    cpu.program_counter.increase(1);
    assert_eq!(cpu.status_register.is_overflow_bit_set(), false);
    assert_eq!(cpu.program_counter.get(), 1);
  }

  #[test_case(random())]
  fn lda(val: u8) {
    let mut cpu = CPU::new();
    cpu.lda(val);
    cpu.program_counter.increase(2);
    assert_eq!(cpu.accumulator.get(), val);
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(random(), r_lsb())]
  fn lda_zero_page(val: u8, index: u8) {
    let mut cpu = setup_zp(0, index, val);
    cpu.zero_page("LDA", &mut CPU::lda);
    assert_eq!(cpu.accumulator.get(), val);
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(random(), random(), r_lsb())]
  fn lda_zero_page_x(val: u8, x: u8, index: u8) {
    let mut cpu = setup_zp_reg(0, index, x, val);
    cpu.x_register.set(x);
    cpu.zp_reg("LDA", x, &mut CPU::lda);
    assert_eq!(cpu.accumulator.get(), val);
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(r_u16(), random())]
  fn lda_absolute(index: u16, val: u8) {
    let mut cpu = setup_abs(index, val);
    cpu.absolute("LDA", &mut CPU::lda);
    assert_eq!(cpu.accumulator.get(), val);
    assert_eq!(cpu.program_counter.get(), 3);
  }

  #[test_case(r_u16(), random(), random())]
  fn lda_absolute_x(index: u16, val: u8, x: u8) {
    let mut cpu = setup_abs_reg(index, x, val);
    cpu.x_register.set(x);
    cpu.absolute_x("LDA", &mut CPU::lda);
    assert_eq!(cpu.accumulator.get(), val);
    assert_eq!(cpu.program_counter.get(), 3);
  }

  #[test_case(r_u16(), random(), random())]
  fn lda_absolute_y(index: u16, y: u8, val: u8) {
    let mut cpu = setup_abs_reg(index, y, val);
    cpu.y_register.set(y);
    cpu.absolute_y("LDA", &mut CPU::lda);
    assert_eq!(cpu.accumulator.get(), val);
    assert_eq!(cpu.program_counter.get(), 3);
  }

  #[test_case(random())]
  fn ldx(val: u8) {
    let mut cpu = CPU::new();
    cpu.ldx(val);
    cpu.program_counter.increase(2);
    assert_eq!(cpu.x_register.get(), val);
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(r_lsb(), random())]
  fn ldx_zero_page(index: u8, val: u8) {
    let mut cpu = setup_zp(0, index, val);
    cpu.zero_page("LDX", &mut CPU::ldx);
    assert_eq!(cpu.x_register.get(), val);
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(r_lsb(), random(), random())]
  fn ldx_zero_page_y(index: u8, val: u8, y: u8) {
    let mut cpu = setup_zp_reg(0, index, y, val);
    cpu.y_register.set(y);
    cpu.zp_reg("LDX", y, &mut CPU::ldx);
    assert_eq!(cpu.x_register.get(), val);
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(r_u16(), random())]
  fn ldx_absolute(index: u16, val: u8) {
    let mut cpu = setup_abs(index, val);
    cpu.absolute("LDX", &mut CPU::ldx);
    assert_eq!(cpu.x_register.get(), val);
    assert_eq!(cpu.program_counter.get(), 3);
  }

  #[test_case(r_u16(), random(), random())]
  fn ldx_absolute_y(index: u16, val: u8, y: u8) {
    let mut cpu = setup_abs_reg(index, y, val);
    cpu.y_register.set(y);
    cpu.absolute_y("LDX", &mut CPU::ldx);
    assert_eq!(cpu.x_register.get(), val);
    assert_eq!(cpu.program_counter.get(), 3);
  }

  #[test_case(random())]
  fn ldy(val: u8) {
    let mut cpu = CPU::new();
    cpu.ldy(val);
    cpu.program_counter.increase(2);
    assert_eq!(cpu.y_register.get(), val);
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(r_lsb(), random())]
  fn ldy_zero_page(index: u8, val: u8) {
    let mut cpu = setup_zp(0, index, val);
    cpu.zero_page("LDY", &mut CPU::ldy);
    assert_eq!(cpu.y_register.get(), val);
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(r_lsb(), random(), random())]
  fn ldy_zero_page_x(index: u8, val: u8, x: u8) {
    let mut cpu = setup_zp_reg(0, index, x, val);
    cpu.x_register.set(x);
    cpu.zp_reg("LDX", x, &mut CPU::ldy);
    assert_eq!(cpu.y_register.get(), val);
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(r_u16(), random())]
  fn ldy_absolute(index: u16, val: u8) {
    let mut cpu = setup_abs(index, val);
    cpu.absolute("LDY", &mut CPU::ldy);
    assert_eq!(cpu.y_register.get(), val);
    assert_eq!(cpu.program_counter.get(), 3);
  }

  #[test_case(r_u16(), random(), random())]
  fn ldy_absolute_x(index: u16, val: u8, x: u8) {
    let mut cpu = setup_abs_reg(index, x, val);
    cpu.x_register.set(x);
    cpu.absolute_x("LDY", &mut CPU::ldy);
    assert_eq!(cpu.y_register.get(), val);
    assert_eq!(cpu.program_counter.get(), 3);
  }

  #[test_case(random(), r_lsb())]
  fn sta_zero_page(val: u8, index: u8) {
    let mut cpu = setup_zp(val, index, 0);
    cpu.sta_zero_page();
    assert_eq!(cpu.memory.get_zero_page(index), val);
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(random(), r_lsb(), random())]
  fn sta_zero_page_x(val: u8, index: u8, x: u8) {
    let mut cpu = setup_zp_reg(val, index, x, 0);
    cpu.x_register.set(x);
    cpu.sta_zero_page_x();
    assert_eq!(cpu.memory.get_zero_page(index.wrapping_add(x)), val);
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(random(), r_u16())]
  fn sta_absolute(val: u8, index: u16) {
    let mut cpu = setup_abs(index, 0);
    cpu.accumulator.set(val);
    cpu.sta_absolute();
    assert_eq!(cpu.memory.get_u16(index), val);
    assert_eq!(cpu.program_counter.get(), 3);
  }

  #[test_case(random(), r_u16(), random())]
  fn sta_absolute_x(val: u8, index: u16, x: u8) {
    let mut cpu = setup_abs_reg(index, x, 0);
    cpu.accumulator.set(val);
    cpu.x_register.set(x);
    cpu.sta_absolute_x();
    assert_eq!(cpu.memory.get_u16(index.wrapping_add(x as u16)), val);
    assert_eq!(cpu.program_counter.get(), 3);
  }

  #[test_case(random(), r_u16(), random())]
  fn sta_absolute_y(val: u8, index: u16, y: u8) {
    let mut cpu = setup_abs_reg(index, y, 0);
    cpu.accumulator.set(val);
    cpu.y_register.set(y);
    cpu.sta_absolute_y();
    assert_eq!(cpu.memory.get_u16(index.wrapping_add(y as u16)), val);
    assert_eq!(cpu.program_counter.get(), 3);
  }

  #[test_case(random(), random(), random(), random(), random())]
  fn sta_indexed_x(val: u8, x: u8, op: u8, v1: u8, v2: u8) {
    let mut cpu = setup_indexed_x(val, 0, v1, v2, op, x);
    cpu.x_register.set(x);
    cpu.sta_indexed_x();
    let index = u16::from_le_bytes([v1, v2]);
    assert_eq!(cpu.memory.get_u16(index), val);
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test_case(random(), random(), random(), random(), random())]
  fn sta_indexed_y(val: u8, y: u8, op: u8, v1: u8, v2: u8) {
    let mut cpu = setup_indexed_y(val, 0, v1, v2, op, y);
    cpu.y_register.set(y);
    let index = u16::from_le_bytes([v1, v2]);
    cpu.sta_indexed_y();
    assert_eq!(cpu.memory.get_u16(index.wrapping_add(y as u16)), val);
    assert_eq!(cpu.program_counter.get(), 2);
  }

  #[test]
  fn sec() {
    let mut cpu = CPU::new();
    cpu.sec();
    cpu.program_counter.increase(1);
    assert_eq!(cpu.status_register.is_carry_bit_set(), true);
    assert_eq!(cpu.program_counter.get(), 1);
  }

  #[test]
  fn sed() {
    let mut cpu = CPU::new();
    cpu.sed();
    cpu.program_counter.increase(1);
    assert_eq!(cpu.status_register.is_decimal_bit_set(), true);
    assert_eq!(cpu.program_counter.get(), 1);
  }

  #[test]
  fn sei() {
    let mut cpu = CPU::new();
    cpu.sei();
    cpu.program_counter.increase(1);
    assert_eq!(cpu.status_register.is_interrupt_bit_set(), true);
    assert_eq!(cpu.program_counter.get(), 1);
  }
}
