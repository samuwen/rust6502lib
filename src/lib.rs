mod memory;
mod registers;

use log::{trace, warn};
use memory::Memory;
use registers::{GeneralRegister, ProgramCounter, StackPointer, StatusRegister};
use std::fmt::{Display, Formatter, Result};

pub const STARTING_MEMORY_BLOCK: u16 = 0x8000;

// TODO: Memory space is 256 pages of 256 bytes. If a page index (hi byte) is incremented
// Cycles should be incremented as well per "page boundary crossing"

/// An emulated CPU for the 6502 processor.
pub struct CPU {
  program_counter: ProgramCounter,
  accumulator: GeneralRegister,
  x_register: GeneralRegister,
  y_register: GeneralRegister,
  status_register: StatusRegister,
  memory: Memory,
  reset_pin: bool,
  nmi_pin: bool,
  irq_pin: bool,
  clock_pin: bool,
}

impl CPU {
  /// Initializes a new CPU instance. Sets all values to 0 by default.
  pub fn new() -> CPU {
    trace!("Initializing CPU");
    CPU {
      program_counter: ProgramCounter::new(),
      accumulator: GeneralRegister::new(),
      x_register: GeneralRegister::new(),
      y_register: GeneralRegister::new(),
      status_register: StatusRegister::new(),
      memory: Memory::new(),
      reset_pin: false,
      nmi_pin: false,
      irq_pin: false,
      clock_pin: false,
    }
  }

  /// Resets the CPU to its initial state. Zeroes everything out basically.
  pub fn reset(&mut self) {
    self.program_counter.reset();
    self.accumulator.reset();
    self.x_register.reset();
    self.y_register.reset();
    self.status_register.reset();
    self.memory.reset();
    self.reset_pin = false;
    self.irq_pin = false;
    self.nmi_pin = false;
    self.clock_pin = false;
    trace!("CPU Reset")
  }

  fn load_program_into_memory(&mut self, program: &Vec<u8>) {
    let mut memory_address = STARTING_MEMORY_BLOCK;
    for byte in program.iter() {
      self.memory.set(memory_address, *byte);
      memory_address += 1;
    }
  }

  /// Waits for a timing signal to be available at the clock pin.
  fn sync(&mut self) {
    while !self.clock_pin {
      self.check_pins();
    }
    self.clock_pin = false;
  }

  /// Simulates signal to the clock pin, enabling the next cycle to execute.
  pub fn tick(&mut self) {
    self.clock_pin = true;
  }

  pub fn set_reset(&mut self) {
    self.reset_pin = true;
  }

  pub fn set_nmi(&mut self) {
    self.nmi_pin = true;
  }

  pub fn set_irq(&mut self) {
    self.irq_pin = true;
  }

  fn check_pins(&mut self) {
    if self.reset_pin {
      self.reset_interrupt();
    }
    if self.nmi_pin {
      self.nmi_interrupt();
    }
    if self.irq_pin && !self.status_register.is_break_bit_set() {
      self.irq_interrupt();
    }
  }

  fn push_to_stack(&mut self, value: u8) {
    self.memory.push_to_stack(value);
    self.sync();
  }

  fn pop_from_stack(&mut self) -> u8 {
    let val = self.memory.pop_from_stack();
    self.sync();
    val
  }

  fn get_u16(&mut self, index: u16) -> u8 {
    let val = self.memory.get_u16(index);
    self.sync();
    val
  }

  fn set_u16(&mut self, index: u16, value: u8) {
    self.memory.set(index, value);
    self.sync();
  }

  fn get_zero_page(&mut self, index: u8) -> u8 {
    let val = self.memory.get_zero_page(index);
    self.sync();
    val
  }

  fn set_zero_page(&mut self, index: u8, value: u8) {
    self.memory.set_zero_page(index, value);
    self.sync();
  }

  /// Used to get the x register when it costs a cycle. This is inconsistent
  /// and I'm sure is kind of wrong.
  fn get_x_register(&mut self) -> u8 {
    let val = self.x_register.get();
    self.sync();
    val
  }

  fn get_single_operand(&mut self) -> u8 {
    let op = self.memory.get_u16(self.program_counter.get_and_increase());
    self.sync();
    op
  }

  fn get_two_operands(&mut self) -> [u8; 2] {
    let lo = self.memory.get_u16(self.program_counter.get_and_increase());
    self.sync();
    let hi = self.memory.get_u16(self.program_counter.get_and_increase());
    self.sync();
    [lo, hi]
  }

  fn test_for_overflow(&mut self, op1: u8, op2: u8) {
    let (_, overflow) = op1.overflowing_add(op2);
    if overflow {
      self.sync();
    }
  }

  fn jump(&mut self, index: u16) {
    self.program_counter.jump(index);
    self.sync();
  }

  /// Runs a program while there are opcodes to handle. This will change when we actually have
  /// a real data set to operate against.
  pub fn run(&mut self, program: Vec<u8>) {
    self.load_program_into_memory(&program);
    loop {
      let opcode = self.get_single_operand();
      match opcode {
        0x24 => self.zero_page_cb("BIT", &mut Self::bit),
        0x29 => self.immediate_cb("AND", &mut Self::and),
        0x2C => self.absolute_cb("BIT", &mut Self::bit),
        0x61 => self.indexed_x_cb("ADC", &mut Self::adc),
        0x65 => self.zero_page_cb("ADC", &mut Self::adc),
        0x69 => self.immediate_cb("ADC", &mut Self::adc),
        0x6D => self.absolute_cb("ADC", &mut Self::adc),
        0x71 => self.indexed_y_cb("ADC", &mut Self::adc),
        0x75 => {
          let x_val = self.get_x_register();
          self.zp_reg_cb("ADC", x_val, &mut Self::adc)
        }
        0x79 => self.absolute_x_cb("ADC", &mut Self::adc),
        0x7D => self.absolute_y_cb("ADC", &mut Self::adc),
        _ => (),
      }
    }
  }

  /*
  ============================================================================================
                                  Generic operations
  ============================================================================================
  */

  fn immediate(&mut self, name: &str) -> u8 {
    let op = self.get_single_operand();
    trace!("{} immediate called with operand:0x{:X}", name, op);
    self.sync();
    op
  }

  fn immediate_cb<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let op = self.immediate(name);
    cb(self, op);
  }

  fn zero_page(&mut self, name: &str) -> (u8, u8) {
    let index = self.get_single_operand();
    trace!("{} zero page called with index: 0x{:X}", name, index);
    (index, self.get_zero_page(index))
  }

  fn zero_page_cb<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let (_, value) = self.zero_page(name);
    cb(self, value);
  }

  fn zp_reg(&mut self, name: &str, reg_val: u8) -> (u8, u8) {
    let op = self.get_single_operand();
    trace!("{} zero page x called with operand: 0x{:X}", name, op);
    let index = op.wrapping_add(reg_val);
    (index, self.get_zero_page(index))
  }

  fn zp_reg_cb<F: FnMut(&mut Self, u8)>(&mut self, name: &str, reg_val: u8, cb: &mut F) {
    let (_, value) = self.zp_reg(name, reg_val);
    cb(self, value);
  }

  fn absolute(&mut self, name: &str) -> (u16, u8) {
    let ops = self.get_two_operands();
    let index = u16::from_le_bytes(ops);
    trace!("{} absolute called with index: 0x{:X}", name, index);
    (index, self.get_u16(index))
  }

  fn absolute_cb<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let (_, value) = self.absolute(name);
    cb(self, value);
  }

  fn absolute_reg(&mut self, name: &str, reg: u8) -> (u16, u8) {
    let ops = self.get_two_operands();
    let index = u16::from_le_bytes(ops);
    trace!("{} absolute reg called with index: 0x{:X}", name, index);
    let total = index.wrapping_add(reg as u16);
    self.test_for_overflow(ops[1], reg);
    (index, self.get_u16(total))
  }

  fn absolute_x_cb<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let (_, value) = self.absolute_reg(name, self.x_register.get());
    cb(self, value);
  }

  fn absolute_y_cb<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let (_, value) = self.absolute_reg(name, self.y_register.get());
    cb(self, value);
  }

  /// AKA Indexed indirect AKA pre-indexed
  fn indexed_x(&mut self, name: &str) -> (u16, u8) {
    let op = self.get_single_operand();
    trace!("{} indexed x called with operand: 0x{:X}", name, op);
    let x_val = self.get_x_register();
    let modified_op = op.wrapping_add(x_val);
    let lo = self.get_zero_page(modified_op);
    let hi = self.get_zero_page(modified_op.wrapping_add(1));
    let index = u16::from_le_bytes([lo, hi]);
    (index, self.get_u16(index))
  }

  fn indexed_x_cb<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let (_, value) = self.indexed_x(name);
    cb(self, value);
  }

  /// AKA Indirect indexed AKA post-indexed
  fn indexed_y(&mut self, name: &str) -> (u16, u8) {
    let op = self.get_single_operand();
    trace!("{} indexed y called with operand: 0x{:X}", name, op);
    let y_val = self.y_register.get();
    let lo = self.get_zero_page(op);
    let hi = self.get_zero_page(op.wrapping_add(1));
    let index = u16::from_le_bytes([lo, hi]);
    self.test_for_overflow(hi, y_val);
    (index, self.memory.get_u16(index))
  }

  fn indexed_y_cb<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let (_, value) = self.indexed_y(name);
    cb(self, value);
  }

  fn flag_operation<F: FnMut(&mut StatusRegister)>(&mut self, name: &str, cb: &mut F) {
    trace!("{} called", name);
    cb(&mut self.status_register);
  }

  fn branch(&mut self, condition: bool, op: u8) {
    if condition {
      let overflow;
      if op > 0x7F {
        overflow = self.program_counter.decrease(!op + 1);
      } else {
        overflow = self.program_counter.increase(op);
      }
      if overflow {
        self.sync();
      }
      self.sync();
    }
  }

  fn generic_compare(&mut self, test_value: u8, reg_value: u8) {
    let (result, carry) = reg_value.overflowing_sub(test_value);
    if result == 0 {
      self.status_register.set_zero_bit();
    }
    if !carry {
      self.status_register.set_carry_bit();
    }
    if (result & 0x80) > 0 {
      self.status_register.set_negative_bit();
    }
  }

  fn register_operation(&mut self, value: u8, message: &str) {
    trace!("{} called with value: {}", message, value);
    self.status_register.handle_n_flag(value, message);
    self.status_register.handle_z_flag(value, message);
  }

  /*
  ============================================================================================
                                  Interrupts
  ============================================================================================
  */

  fn interrupt(&mut self, low_vec: u16, hi_vec: u16) -> u16 {
    self.internal_operations();
    let ops = self.program_counter.to_le_bytes();
    self.push_to_stack(ops[0]);
    self.push_to_stack(ops[1]);
    self.push_to_stack(self.status_register.get_register());
    let lo = self.get_u16(low_vec);
    let hi = self.get_u16(hi_vec);
    u16::from_le_bytes([lo, hi])
  }

  fn return_from_interrupt(&mut self) {
    self.internal_operations();
    let status_reg = self.pop_from_stack();
    self.status_register.set(status_reg);
    let hi_pc = self.pop_from_stack();
    let lo_pc = self.pop_from_stack();
    self.jump(u16::from_le_bytes([lo_pc, hi_pc]));
  }

  /// Unspecified thing that delays execution by two cycles.
  fn internal_operations(&mut self) {
    self.sync();
    self.sync();
  }

  /// Resets the system. Some data will be left over after depending on where
  /// the program was in the execution cycle
  fn reset_interrupt(&mut self) {
    let index = self.interrupt(0xFFFC, 0xFFFD);
    self.program_counter.jump(index);
    self.reset();
  }

  fn nmi_interrupt(&mut self) {
    let index = self.interrupt(0xFFFA, 0xFFFB);
    self.program_counter.jump(index);
  }

  fn irq_interrupt(&mut self) {
    let index = self.interrupt(0xFFFE, 0xFFFF);
    self.program_counter.jump(index);
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
    let message = "ASL";
    trace!("{} called with value: 0x{:X}", message, value);
    let (result, carry) = value.overflowing_shl(1);
    self.sync();
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_z_flag(result, message);
    self.status_register.handle_c_flag(message, carry);
    result
  }

  pub fn asl_accumulator(&mut self) {
    let result = self.asl(self.accumulator.get());
    self.accumulator.set(result);
  }

  pub fn asl_zero_page(&mut self) {
    let (index, value) = self.zero_page("ASL");
    let result = self.asl(value);
    self.set_zero_page(index, result);
  }

  pub fn asl_zero_page_x(&mut self) {
    let x_val = self.get_x_register();
    let (index, value) = self.zp_reg("ASL", x_val);
    let result = self.asl(value);
    self.set_zero_page(index, result);
  }

  pub fn asl_absolute(&mut self) {
    let (index, value) = self.absolute("ASL");
    let result = self.asl(value);
    self.set_u16(index, result);
  }

  pub fn asl_absolute_x(&mut self) {
    let x_val = self.get_x_register();
    let (index, value) = self.absolute_reg("ASL", x_val);
    let result = self.asl(value);
    self.set_u16(index, result);
  }

  /// Tests a value and sets flags accordingly.
  ///
  /// Zero is set by looking at the result of the value AND the accumulator.
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

  pub fn bpl(&mut self) {
    let op = self.get_single_operand();
    self.branch(!self.status_register.is_negative_bit_set(), op);
  }

  pub fn bmi(&mut self) {
    let op = self.get_single_operand();
    self.branch(self.status_register.is_negative_bit_set(), op);
  }

  pub fn bvc(&mut self) {
    let op = self.get_single_operand();
    self.branch(!self.status_register.is_overflow_bit_set(), op);
  }

  pub fn bvs(&mut self) {
    let op = self.get_single_operand();
    self.branch(self.status_register.is_overflow_bit_set(), op);
  }

  pub fn bcc(&mut self) {
    let op = self.get_single_operand();
    self.branch(!self.status_register.is_carry_bit_set(), op);
  }

  pub fn bcs(&mut self) {
    let op = self.get_single_operand();
    self.branch(self.status_register.is_carry_bit_set(), op);
  }

  pub fn bne(&mut self) {
    let op = self.get_single_operand();
    self.branch(!self.status_register.is_zero_bit_set(), op);
  }

  pub fn beq(&mut self) {
    let op = self.get_single_operand();
    self.branch(self.status_register.is_zero_bit_set(), op);
  }

  pub fn brk(&mut self) {
    self.program_counter.increment();
    self.nmi_interrupt();
  }

  pub fn cmp(&mut self, test_value: u8) {
    self.generic_compare(test_value, self.accumulator.get());
  }

  pub fn cpx(&mut self, test_value: u8) {
    self.generic_compare(test_value, self.x_register.get());
  }

  pub fn cpy(&mut self, test_value: u8) {
    self.generic_compare(test_value, self.y_register.get());
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

  pub fn dec(&mut self, index: u16) {
    let value = self.get_u16(index);
    let value = value.wrapping_sub(1);
    trace!("DEC called index: {}, value: {}", index, value);
    self.set_u16(index, value);
    self.status_register.handle_n_flag(value, "DEC");
    self.status_register.handle_z_flag(value, "DEC");
  }

  pub fn dec_zp(&mut self) {
    let (index, _) = self.zero_page("DEC");
    self.dec(index as u16);
  }

  pub fn dec_zp_reg(&mut self) {
    let x_val = self.get_x_register();
    let (index, _) = self.zp_reg("DEC", x_val);
    self.dec(index as u16);
  }

  pub fn dec_abs(&mut self) {
    let (index, _) = self.absolute("DEC");
    self.dec(index as u16);
  }

  pub fn dec_abs_x(&mut self) {
    let (index, _) = self.absolute_reg("DEC", self.x_register.get());
    self.dec(index as u16);
  }

  /// DEDICATED TO XOR - GOD OF INVERSE
  pub fn eor(&mut self, value: u8) {
    let message = "EOR";
    trace!("{} called with value: 0x{:X}", message, value);
    let result = self.accumulator.get() ^ value;
    self.accumulator.set(result);
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_z_flag(result, message);
  }

  pub fn inc(&mut self, index: u16) {
    let value = self.memory.get_u16(index);
    let value = value.wrapping_add(1);
    trace!("INC called index: {}, value: {}", index, value);
    self.memory.set(index, value);
    self.status_register.handle_n_flag(value, "INC");
    self.status_register.handle_z_flag(value, "INC");
  }

  pub fn inc_zp(&mut self) {
    let (index, _) = self.zero_page("INC");
    self.inc(index as u16);
  }

  pub fn inc_zp_reg(&mut self) {
    let (index, _) = self.zp_reg("INC", self.x_register.get());
    self.inc(index as u16);
  }

  pub fn inc_abs(&mut self) {
    let (index, _) = self.absolute("INC");
    self.inc(index as u16);
  }

  pub fn inc_abs_x(&mut self) {
    let (index, _) = self.absolute_reg("INC", self.x_register.get());
    self.inc(index as u16);
  }

  pub fn jmp_absolute(&mut self) {
    let ops = self.get_two_operands();
    let index = u16::from_le_bytes(ops);
    trace!("JMP absolute to index: {}", index);
    self.program_counter.jump(index);
  }

  pub fn jmp_indirect(&mut self) {
    let ops = self.get_two_operands();
    let (low_test, overflow) = ops[0].overflowing_add(1);
    if overflow {
      warn!("Indirect jump overflowing page. Results will be weird!");
    }
    let hi = self.memory.get_u16(u16::from_le_bytes(ops));
    let lo = self.memory.get_u16(u16::from_le_bytes([low_test, ops[1]]));
    let index = u16::from_le_bytes([hi, lo]);
    trace!("JMP indirect to index: {}", index);
    self.program_counter.jump(index);
  }

  pub fn jsr(&mut self) {
    let ops = self.get_two_operands();
    self.program_counter.decrease(1);
    let pc_ops = self.program_counter.get().to_le_bytes();
    self.memory.push_to_stack(pc_ops[0]);
    self.memory.push_to_stack(pc_ops[1]);
    let index = u16::from_le_bytes(ops);
    trace!("JSR to index: {}, PC stored on stack", index,);
    self.program_counter.jump(index);
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

  /// Shifts all bits right one position
  ///
  /// Affects flags N Z C
  fn lsr(&mut self, value: u8) -> u8 {
    let (result, carry) = value.overflowing_shr(1);
    self.status_register.handle_n_flag(result, "LSR");
    self.status_register.handle_z_flag(result, "LSR");
    self.status_register.handle_c_flag("LSR", carry);
    result
  }

  pub fn lsr_accumulator(&mut self) {
    let result = self.lsr(self.accumulator.get());
    self.accumulator.set(result);
  }

  pub fn lsr_zero_page(&mut self) {
    let (index, value) = self.zero_page("LSR");
    let result = self.lsr(value);
    self.memory.set_zero_page(index, result);
  }

  pub fn lsr_zero_page_x(&mut self) {
    let (index, value) = self.zp_reg("LSR", self.x_register.get());
    let result = self.lsr(value);
    self.memory.set_zero_page(index, result);
  }

  pub fn lsr_absolute(&mut self) {
    let (index, value) = self.absolute("LSR");
    let result = self.lsr(value);
    self.memory.set(index, result);
  }

  pub fn lsr_absolute_x(&mut self) {
    let (index, value) = self.absolute_reg("LSR", self.x_register.get());
    let result = self.lsr(value);
    self.memory.set(index, result);
  }

  pub fn nop(&mut self) {
    // do nothing - but take cycle time
  }

  pub fn ora(&mut self, value: u8) {
    let message = "ORA";
    trace!("{} called with value: 0x{:X}", message, value);
    let result = self.accumulator.get() | value;
    self.accumulator.set(result);
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_z_flag(result, message);
  }

  pub fn tax(&mut self) {
    self.x_register.set(self.accumulator.get());
    self.register_operation(self.x_register.get(), "TAX");
  }

  pub fn txa(&mut self) {
    self.accumulator.set(self.x_register.get());
    self.register_operation(self.x_register.get(), "TXA");
  }

  pub fn dex(&mut self) {
    self.x_register.decrement();
    self.register_operation(self.x_register.get(), "DEX");
  }

  pub fn inx(&mut self) {
    self.x_register.increment();
    self.register_operation(self.x_register.get(), "INX");
  }

  pub fn tay(&mut self) {
    self.y_register.set(self.accumulator.get());
    self.register_operation(self.y_register.get(), "TAY");
  }

  pub fn tya(&mut self) {
    self.accumulator.set(self.y_register.get());
    self.register_operation(self.y_register.get(), "TYA");
  }

  pub fn dey(&mut self) {
    self.y_register.decrement();
    self.register_operation(self.y_register.get(), "DEY");
  }

  pub fn iny(&mut self) {
    self.y_register.increment();
    self.register_operation(self.y_register.get(), "INY");
  }

  fn rol(&mut self, value: u8) -> u8 {
    trace!("ROL called with value: {}", value);
    let (mut result, carry) = value.overflowing_shl(1);
    if self.status_register.is_carry_bit_set() {
      result |= 0x1;
    }
    if carry {
      self.status_register.set_carry_bit();
    }
    self.status_register.handle_n_flag(result, "ROL");
    self.status_register.handle_z_flag(result, "ROL");
    result
  }

  pub fn rol_accumulator(&mut self) {
    let result = self.rol(self.accumulator.get());
    self.accumulator.set(result);
  }

  pub fn rol_zero_page(&mut self) {
    let (index, value) = self.zero_page("ROL");
    let result = self.rol(value);
    self.memory.set_zero_page(index, result);
  }

  pub fn rol_zero_page_x(&mut self) {
    let (index, value) = self.zp_reg("ROL", self.x_register.get());
    let result = self.rol(value);
    self.memory.set_zero_page(index, result);
  }

  pub fn rol_absolute(&mut self) {
    let (index, value) = self.absolute("ROL");
    let result = self.rol(value);
    self.memory.set(index, result);
  }

  pub fn rol_absolute_x(&mut self) {
    let (index, value) = self.absolute_reg("ROL", self.x_register.get());
    let result = self.rol(value);
    self.memory.set(index, result);
  }

  fn ror(&mut self, value: u8) -> u8 {
    trace!("ROR called with value: {}", value);
    let (mut result, carry) = value.overflowing_shl(1);
    if self.status_register.is_carry_bit_set() {
      result |= 0x1;
    }
    if carry {
      self.status_register.set_carry_bit();
    }
    self.status_register.handle_n_flag(result, "ROR");
    self.status_register.handle_z_flag(result, "ROR");
    result
  }

  pub fn ror_accumulator(&mut self) {
    let result = self.ror(self.accumulator.get());
    self.accumulator.set(result);
  }

  pub fn ror_zero_page(&mut self) {
    let (index, value) = self.zero_page("ROR");
    let result = self.ror(value);
    self.memory.set_zero_page(index, result);
  }

  pub fn ror_zero_page_x(&mut self) {
    let (index, value) = self.zp_reg("ROR", self.x_register.get());
    let result = self.ror(value);
    self.memory.set_zero_page(index, result);
  }

  pub fn ror_absolute(&mut self) {
    let (index, value) = self.absolute("ROR");
    let result = self.ror(value);
    self.memory.set(index, result);
  }

  pub fn ror_absolute_x(&mut self) {
    let (index, value) = self.absolute_reg("ROR", self.x_register.get());
    let result = self.ror(value);
    self.memory.set(index, result);
  }

  /// Return from interrupt
  pub fn rti(&mut self) {
    self.return_from_interrupt();
  }

  pub fn rts(&mut self) {
    let lo = self.memory.pop_from_stack();
    let hi = self.memory.pop_from_stack();
    let index = u16::from_le_bytes([lo, hi]) + 1;
    self.program_counter.jump(index);
  }

  pub fn sbc(&mut self, value: u8) {
    let message = "SBC";
    trace!("{} called with value: 0x{:X}", message, value);
    let (result, carry) = self.accumulator.get().overflowing_sub(value);
    self.accumulator.set(result);
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_v_flag(result, message, carry);
    self.status_register.handle_z_flag(result, message);
    self.status_register.handle_c_flag(message, carry);
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
    let (index, _) = self.zero_page("STA");
    self.memory.set_zero_page(index, self.accumulator.get());
  }

  pub fn sta_zero_page_x(&mut self) {
    let (index, _) = self.zp_reg("STA", self.x_register.get());
    self.memory.set_zero_page(index, self.accumulator.get());
  }

  pub fn sta_absolute(&mut self) {
    let (index, _) = self.absolute("STA");
    self.memory.set(index, self.accumulator.get());
  }

  pub fn sta_absolute_x(&mut self) {
    let (index, _) = self.absolute_reg("STA", self.x_register.get());
    self.memory.set(index, self.accumulator.get());
  }

  pub fn sta_absolute_y(&mut self) {
    let (index, _) = self.absolute_reg("STA", self.y_register.get());
    self.memory.set(index, self.accumulator.get());
  }

  pub fn sta_indexed_x(&mut self) {
    let (index, _) = self.indexed_x("STA");
    self.memory.set(index, self.accumulator.get());
  }

  pub fn sta_indexed_y(&mut self) {
    let (index, _) = self.indexed_y("STA");
    self.memory.set(index, self.accumulator.get());
  }

  pub fn txs(&mut self) {
    self.memory.set_stack_pointer(self.x_register.get());
  }

  pub fn tsx(&mut self) {
    self.x_register.set(self.memory.get_stack_pointer().get());
  }

  pub fn pha(&mut self) {
    self.memory.push_to_stack(self.accumulator.get());
  }

  pub fn pla(&mut self) {
    self.accumulator.set(self.memory.pop_from_stack());
  }

  pub fn php(&mut self) {
    self
      .memory
      .push_to_stack(self.status_register.get_register());
  }

  pub fn plp(&mut self) {
    self.status_register.set(self.memory.pop_from_stack());
  }

  pub fn stx_zero_page(&mut self) {
    let (index, _) = self.zero_page("STX");
    self.memory.set_zero_page(index, self.x_register.get());
  }

  pub fn stx_zero_page_y(&mut self) {
    let (index, _) = self.zp_reg("STX", self.y_register.get());
    self.memory.set_zero_page(index, self.x_register.get());
  }

  pub fn stx_absolute(&mut self) {
    let (index, _) = self.absolute("STX");
    self.memory.set(index, self.x_register.get());
  }

  pub fn sty_zero_page(&mut self) {
    let (index, _) = self.zero_page("STY");
    self.memory.set_zero_page(index, self.y_register.get());
  }

  pub fn sty_zero_page_y(&mut self) {
    let (index, _) = self.zp_reg("STY", self.x_register.get());
    self.memory.set_zero_page(index, self.y_register.get());
  }

  pub fn sty_absolute(&mut self) {
    let (index, _) = self.absolute("STY");
    self.memory.set(index, self.y_register.get());
  }
}

impl Display for CPU {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(
      f,
      "program_counter: 0x{:X}\nstack_pointer: 0x{:X}\naccumulator: 0x{:X}\nstatus_register: {}\nx_register: 0x{:X}\ny_register: 0x{:X}\n",
      self.program_counter.get(), self.memory.get_stack_pointer().get(), self.accumulator.get(), self.status_register, self.x_register.get(), self.y_register.get()
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
}
