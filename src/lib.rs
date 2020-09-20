mod memory;
mod registers;

use log::{debug, trace, warn};
use memory::Memory;
use registers::{GeneralRegister, ProgramCounter, StackPointer, StatusBit, StatusRegister};
use std::fmt::{Display, Formatter, Result};

/// A semi-arbitrary choice for where to start program execution. This is what the NES uses
/// so I figured its as good a place as any to begin.
pub const STARTING_MEMORY_BLOCK: u16 = 0x8000;

/// An emulated CPU for the 6502 processor.
///
/// The 6502 is a little endian machine.
/// The 6502 has 3 general purpose registers, X and Y, and an Accumulator.
/// It has a program counter to keep track of program execution.
/// It has a status register to keep track of 7 different status flags.
/// It has onboard 16 bit memory.
/// We simulate 4 hardware pins - 3 for interrupts and one for the clock.
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
  /// Initializes a new CPU instance. Sets all values to their associated defaults.
  pub fn new() -> CPU {
    debug!("Initializing CPU");
    CPU {
      program_counter: ProgramCounter::new(),
      accumulator: GeneralRegister::new(),
      x_register: GeneralRegister::new(),
      y_register: GeneralRegister::new(),
      status_register: StatusRegister::new(),
      memory: Memory::new(),
      clock_pin: false,
      reset_pin: false,
      irq_pin: false,
      nmi_pin: false,
    }
  }

  pub fn reset(&mut self) {
    debug!("Resetting CPU");
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
  }

  /// Loads the program into memory.
  ///
  /// # Panics
  /// Panics if the program is too big for the allocated memory space.
  fn load_program_into_memory(&mut self, program: &Vec<u8>, block: u16) {
    debug!("Loading program into memory starting at: {}", block);
    let mut memory_address = block;
    if program.len() + block as usize > 0xFFFF {
      panic!("Program is too large for allocated memory space. Maybe you didn't set a custom starting block?");
    }
    for byte in program.iter() {
      self.memory.set(memory_address, *byte);
      memory_address += 1;
    }
  }

  /// Waits for a timing signal to be available at the clock pin. Checks interrupt
  /// pins in the meanwhile. This simulates that the cpu will finish its current
  /// instruction when an interrupt comes in, then go off and handle the interrupt.
  fn sync(&mut self) {
    trace!("Completed machine cycle");
    while !self.clock_pin {
      self.check_pins();
    }
    self.clock_pin = false;
    trace!("Starting machine cycle");
  }

  /// Simulates signal to the clock pin, enabling the next cycle to execute.
  pub fn tick(&mut self) {
    self.clock_pin = true;
  }

  /// Sets the reset pin to allow for a reset interrupt.
  pub fn set_reset(&mut self) {
    trace!("Reset pin set. CPU should now reset");
    self.reset_pin = true;
  }

  /// Sets the NMI pin to allow for a Non-Maskable interrupt. Non-maskable interrupts
  /// are run regardless of the setting of the interrupt bit.
  pub fn set_nmi(&mut self) {
    trace!("NMI pin set. Interrupt should be handled");
    self.nmi_pin = true;
  }

  /// Sets the IRQ pin to allow for a Maskable interrupt. Maskable interrupts are run
  /// only when the interrupt bit is unset.
  pub fn set_irq(&mut self) {
    trace!("IRQ pin set. Interrupt may be handled");
    self.irq_pin = true;
  }

  /// Checks to see if we have an interrupt at the pins. Check is in priority order,
  /// that being Reset, NMI, IRQ.
  fn check_pins(&mut self) {
    if self.reset_pin {
      self.reset_interrupt();
    }
    if self.nmi_pin {
      self.nmi_interrupt();
    }
    if self.irq_pin && !self.status_register.is_flag_set(StatusBit::Interrupt) {
      self.irq_interrupt();
    }
  }

  /// Pushes a value to the stack. Memory operations cost machine cycles so this
  /// waits for a cycle.
  fn push_to_stack(&mut self, value: u8) {
    trace!("Push to stack wrapper called");
    self.memory.push_to_stack(value);
    // writing to memory
    self.sync();
  }

  /// Pops(pulls) a value from the stack. Memory operations cost machine cycles
  /// so this waits for a cycle. Pop operations (poperations?) also cost a
  /// machine cycle so we account for that as well.
  fn pop_from_stack(&mut self) -> u8 {
    trace!("Pop from stack wrapper called");
    // incrementing the pointer
    self.sync();
    let val = self.memory.pop_from_stack();
    // reading from memory
    self.sync();
    val
  }

  /// Wrapper around getting a 16 bit memory value. We wrap this because memory
  /// operations cost machine cycles so this waits for a cycle.
  fn get_u16(&mut self, index: u16) -> u8 {
    trace!("Get u16 wrapper called");
    let val = self.memory.get_u16(index);
    self.sync();
    val
  }

  /// Wrapper around setting a 16 bit memory value. We wrap this because memory
  /// operations cost machine cycles so this waits for a cycle.
  fn set_u16(&mut self, index: u16, value: u8) {
    trace!("Set u16 wrapper called");
    self.memory.set(index, value);
    self.sync();
  }

  /// Wrapper around setting a zero page value. We wrap this because memory operations
  /// cost machine cycles so this waits for a cycle.
  fn get_zero_page(&mut self, index: u8) -> u8 {
    trace!("Get zero page wrapper called");
    let val = self.memory.get_zero_page(index);
    self.sync();
    val
  }

  /// Wrapper around getting a zero page value. We wrap this because memory operations
  /// cost machine cycles so this waits for a cycle.
  fn set_zero_page(&mut self, index: u8, value: u8) {
    trace!("Set zero page wrapper called");
    self.memory.set_zero_page(index, value);
    self.sync();
  }

  /// Gets a byte from the program under execution. This increments the program counter,
  /// returns the value in memory at the address returned by the counter, and waits for
  /// a cycle.
  fn get_single_operand(&mut self) -> u8 {
    let op = self.memory.get_u16(self.program_counter.get_and_increase());
    debug!("Getting an operand with value: {}", op);
    self.sync();
    op
  }

  /// Gets two bytes from the program under execution. This increments the program counter,
  /// returns the value in memory at the address returned by the counter, and waits for
  /// a cycle, twice.
  fn get_two_operands(&mut self) -> [u8; 2] {
    let lo = self.get_single_operand();
    let hi = self.get_single_operand();
    [lo, hi]
  }

  /// For some addressing modes, if bytes overflow it costs an additional machine cycle. We
  /// validate if an overflow occurred, and delay as appropriate if needed.
  fn test_for_overflow(&mut self, op1: u8, op2: u8) {
    let (_, overflow) = op1.overflowing_add(op2);
    if overflow {
      trace!("Overflow found, costing a machine cycle");
      self.sync();
    }
  }

  /// Loads a program and begins running it.
  ///
  /// Programs must be provided as vectors of byte code, optionally
  /// providing a starting block for where the program should live in memory. This will load
  /// the program into memory starting at the block specified.
  ///
  /// Once the program is loaded, enters a loop that gets the next opcode and matches its number
  /// to the master opcode map, calling the explicit opcode function.
  ///
  /// # Panics
  /// This will panic if the program is larger than the remaining difference between 0xFFFF and
  /// the starting memory block provided.
  ///
  /// # Notes
  /// Official opcodes were built and implemented based off the information at
  /// http://6502.org/tutorials/6502opcodes.html
  /// Illegal opcodes were built and implemented based off the information at
  /// http://nesdev.com/undocumented_opcodes.txt
  pub fn run(&mut self, program: Vec<u8>, start: Option<u16>) {
    let block = match start {
      Some(v) => v,
      None => STARTING_MEMORY_BLOCK,
    };
    self.load_program_into_memory(&program, block);
    debug!("Program loaded. Beginning run loop");
    loop {
      let opcode = self.get_single_operand();
      match opcode {
        0x00 => self.brk(),
        0x01 => self.indexed_x_cb("ORA", &mut Self::ora),
        0x02 => self.kil(),
        0x03 => self.indexed_x_cb("SLO", &mut Self::slo),
        0x04 => self.zero_page_cb("DOP", &mut Self::dop),
        0x05 => self.zero_page_cb("ORA", &mut Self::ora),
        0x06 => self.asl_zero_page(),
        0x07 => self.zero_page_cb("SLO", &mut Self::slo),
        0x08 => self.php(),
        0x09 => self.immediate_cb("ORA", &mut Self::ora),
        0x0A => self.asl_accumulator(),
        0x0B => self.immediate_cb("AAC", &mut Self::aac),
        0x0C => self.absolute_cb("TOP", &mut Self::top),
        0x0D => self.absolute_cb("ORA", &mut Self::ora),
        0x0E => self.asl_absolute(),
        0x0F => self.absolute_cb("SLO", &mut Self::slo),
        0x10 => self.bpl(),
        0x11 => self.indexed_y_cb("ORA", &mut Self::ora),
        0x12 => self.kil(),
        0x13 => self.indexed_y_cb("SLO", &mut Self::slo),
        0x14 => self.zp_reg_cb("DOP", self.x_register.get(), &mut Self::dop),
        0x15 => self.zp_reg_cb("ORA", self.x_register.get(), &mut Self::ora),
        0x16 => self.asl_zero_page_x(),
        0x17 => self.zp_reg_cb("SLO", self.x_register.get(), &mut Self::slo),
        0x18 => self.clc(),
        0x19 => self.absolute_y_cb("ORA", &mut Self::ora),
        0x1A => self.nop(),
        0x1B => self.absolute_y_cb("SLO", &mut Self::slo),
        0x1C => self.absolute_x_cb("TOP", &mut Self::top),
        0x1D => self.absolute_x_cb("ORA", &mut Self::ora),
        0x1E => self.asl_absolute_x(),
        0x1F => self.absolute_x_cb("SLO", &mut Self::slo),
        0x20 => self.jsr(),
        0x21 => self.indexed_x_cb("AND", &mut Self::and),
        0x22 => self.kil(),
        0x23 => self.indexed_x_cb("RLA", &mut Self::rla),
        0x24 => self.zero_page_cb("BIT", &mut Self::bit),
        0x25 => self.zero_page_cb("AND", &mut Self::and),
        0x26 => self.rol_zero_page(),
        0x27 => self.zero_page_cb("RLA", &mut Self::rla),
        0x28 => self.plp(),
        0x29 => self.immediate_cb("AND", &mut Self::and),
        0x2A => self.rol_accumulator(),
        0x2B => self.immediate_cb("AAC", &mut Self::aac),
        0x2C => self.absolute_cb("BIT", &mut Self::bit),
        0x2D => self.absolute_cb("AND", &mut Self::and),
        0x2E => self.rol_absolute(),
        0x2F => self.absolute_cb("RLA", &mut Self::rla),
        0x30 => self.bmi(),
        0x31 => self.indexed_y_cb("AND", &mut Self::and),
        0x32 => self.kil(),
        0x33 => self.indexed_y_cb("RLA", &mut Self::rla),
        0x34 => self.zp_reg_cb("DOP", self.x_register.get(), &mut Self::dop),
        0x35 => self.zp_reg_cb("AND", self.x_register.get(), &mut Self::and),
        0x36 => self.rol_zero_page_x(),
        0x37 => self.zp_reg_cb("RLA", self.x_register.get(), &mut Self::rla),
        0x38 => self.sec(),
        0x39 => self.absolute_y_cb("AND", &mut Self::and),
        0x3A => self.nop(),
        0x3B => self.absolute_y_cb("RLA", &mut Self::rla),
        0x3C => self.absolute_x_cb("TOP", &mut Self::top),
        0x3D => self.absolute_x_cb("AND", &mut Self::and),
        0x3E => self.rol_absolute_x(),
        0x3F => self.absolute_x_cb("RLA", &mut Self::rla),
        0x40 => self.rti(),
        0x41 => self.indexed_x_cb("EOR", &mut Self::eor),
        0x42 => self.kil(),
        0x43 => self.indexed_x_cb("SRE", &mut Self::sre),
        0x44 => self.zero_page_cb("DOP", &mut Self::dop),
        0x45 => self.zero_page_cb("EOR", &mut Self::eor),
        0x46 => self.lsr_zero_page(),
        0x47 => self.zero_page_cb("SRE", &mut Self::sre),
        0x48 => self.pha(),
        0x49 => self.immediate_cb("EOR", &mut Self::eor),
        0x4A => self.lsr_accumulator(),
        0x4B => self.immediate_cb("ASR", &mut Self::asr),
        0x4C => self.jmp_absolute(),
        0x4D => self.absolute_cb("EOR", &mut Self::eor),
        0x4E => self.lsr_absolute(),
        0x4F => self.absolute_cb("SRE", &mut Self::sre),
        0x50 => self.bvc(),
        0x51 => self.indexed_y_cb("EOR", &mut Self::eor),
        0x52 => self.kil(),
        0x53 => self.indexed_y_cb("SRE", &mut Self::sre),
        0x54 => self.zp_reg_cb("DOP", self.x_register.get(), &mut Self::dop),
        0x55 => self.zp_reg_cb("EOR", self.x_register.get(), &mut Self::eor),
        0x56 => self.lsr_zero_page_x(),
        0x57 => self.zp_reg_cb("SRE", self.x_register.get(), &mut Self::sre),
        0x58 => self.cli(),
        0x59 => self.absolute_y_cb("EOR", &mut Self::eor),
        0x5A => self.nop(),
        0x5B => self.absolute_y_cb("SRE", &mut Self::sre),
        0x5C => self.absolute_x_cb("TOP", &mut Self::top),
        0x5D => self.absolute_x_cb("EOR", &mut Self::eor),
        0x5E => self.lsr_absolute_x(),
        0x5F => self.absolute_x_cb("SRE", &mut Self::sre),
        0x60 => self.rts(),
        0x61 => self.indexed_x_cb("ADC", &mut Self::adc),
        0x62 => self.kil(),
        0x63 => self.indexed_x_cb("RRA", &mut Self::rra),
        0x64 => self.zero_page_cb("DOP", &mut Self::dop),
        0x65 => self.zero_page_cb("ADC", &mut Self::adc),
        0x66 => self.ror_zero_page(),
        0x67 => self.zero_page_cb("RRA", &mut Self::rra),
        0x68 => self.pla(),
        0x69 => self.immediate_cb("ADC", &mut Self::adc),
        0x6A => self.ror_accumulator(),
        0x6B => self.immediate_cb("ARR", &mut Self::arr),
        0x6C => self.jmp_indirect(),
        0x6D => self.absolute_cb("ADC", &mut Self::adc),
        0x6E => self.ror_absolute(),
        0x6F => self.absolute_cb("RRA", &mut Self::rra),
        0x70 => self.bvs(),
        0x71 => self.indexed_y_cb("ADC", &mut Self::adc),
        0x72 => self.kil(),
        0x73 => self.indexed_y_cb("RRA", &mut Self::rra),
        0x74 => self.zp_reg_cb("DOP", self.x_register.get(), &mut Self::dop),
        0x75 => self.zp_reg_cb("ADC", self.x_register.get(), &mut Self::adc),
        0x76 => self.ror_zero_page_x(),
        0x77 => self.zp_reg_cb("RRA", self.x_register.get(), &mut Self::rra),
        0x78 => self.sei(),
        0x79 => self.absolute_x_cb("ADC", &mut Self::adc),
        0x7A => self.nop(),
        0x7B => self.absolute_y_cb("RRA", &mut Self::rra),
        0x7C => self.absolute_x_cb("TOP", &mut Self::top),
        0x7D => self.absolute_y_cb("ADC", &mut Self::adc),
        0x7E => self.ror_absolute_x(),
        0x7F => self.absolute_x_cb("RRA", &mut Self::rra),
        0x80 => self.immediate_cb("DOP", &mut Self::dop),
        0x81 => self.sta_indexed_x(),
        0x82 => self.immediate_cb("DOP", &mut Self::dop),
        0x83 => self.aax_indirect_x(),
        0x84 => self.sty_zero_page(),
        0x85 => self.sta_zero_page(),
        0x86 => self.stx_zero_page(),
        0x87 => self.aax_zero_page(),
        0x88 => self.dey(),
        0x89 => self.immediate_cb("DOP", &mut Self::dop),
        0x8A => self.txa(),
        0x8B => self.xaa(),
        0x8C => self.sty_absolute(),
        0x8D => self.sta_absolute(),
        0x8E => self.stx_absolute(),
        0x8F => self.aax_absolute(),
        0x90 => self.bcc(),
        0x91 => self.sta_indexed_y(),
        0x92 => self.kil(),
        0x93 => self.axa_indirect(),
        0x94 => self.sty_zero_page_x(),
        0x95 => self.sta_zero_page_x(),
        0x96 => self.stx_zero_page_y(),
        0x97 => self.aax_zero_page_y(),
        0x98 => self.tya(),
        0x99 => self.sta_absolute_y(),
        0x9A => self.txs(),
        0x9B => self.xas(),
        0x9C => self.sya(),
        0x9D => self.sta_absolute_x(),
        0x9E => self.sxa(),
        0x9F => self.axa_absolute_y(),
        0xA0 => self.immediate_cb("LDY", &mut Self::ldy),
        0xA1 => self.indexed_x_cb("LDA", &mut Self::lda),
        0xA2 => self.immediate_cb("LDX", &mut Self::ldx),
        0xA3 => self.indexed_x_cb("LAX", &mut Self::lax),
        0xA4 => self.zero_page_cb("LDY", &mut Self::ldx),
        0xA5 => self.zero_page_cb("LDA", &mut Self::lda),
        0xA7 => self.zero_page_cb("LAX", &mut Self::lax),
        0xA6 => self.zero_page_cb("LDX", &mut Self::ldx),
        0xA8 => self.tay(),
        0xA9 => self.immediate_cb("LDA", &mut Self::lda),
        0xAA => self.tax(),
        0xAB => self.immediate_cb("ATX", &mut Self::atx),
        0xAC => self.absolute_cb("LDY", &mut Self::ldy),
        0xAD => self.absolute_cb("LDA", &mut Self::lda),
        0xAE => self.absolute_cb("LDX", &mut Self::ldx),
        0xAF => self.absolute_cb("LAX", &mut Self::lax),
        0xB0 => self.bcs(),
        0xB1 => self.indexed_y_cb("LDA", &mut Self::lda),
        0xB2 => self.kil(),
        0xB3 => self.indexed_y_cb("LAX", &mut Self::lax),
        0xB8 => self.clv(),
        0xB4 => self.zp_reg_cb("LDY", self.x_register.get(), &mut Self::ldy),
        0xB5 => self.zp_reg_cb("LDA", self.x_register.get(), &mut Self::lda),
        0xB6 => self.zp_reg_cb("LDX", self.y_register.get(), &mut Self::ldx),
        0xB7 => self.zp_reg_cb("LAX", self.y_register.get(), &mut Self::lax),
        0xB9 => self.absolute_y_cb("LDA", &mut Self::lda),
        0xBA => self.tsx(),
        0xBB => self.absolute_y_cb("LAR", &mut Self::lar),
        0xBC => self.absolute_x_cb("LDY", &mut Self::ldy),
        0xBD => self.absolute_x_cb("LDA", &mut Self::lda),
        0xBE => self.absolute_y_cb("LDX", &mut Self::ldx),
        0xBF => self.absolute_y_cb("LAX", &mut Self::lax),
        0xC0 => self.immediate_cb("CPY", &mut Self::cpy),
        0xC1 => self.indexed_x_cb("CMP", &mut Self::cmp),
        0xC2 => self.immediate_cb("DOP", &mut Self::dop),
        0xC3 => self.dcp_indexed_x(),
        0xC4 => self.zero_page_cb("CPY", &mut Self::cpy),
        0xC5 => self.zero_page_cb("CMP", &mut Self::cmp),
        0xC6 => self.dec_zp(),
        0xC7 => self.dcp_zp(),
        0xC8 => self.iny(),
        0xC9 => self.immediate_cb("CMP", &mut Self::cmp),
        0xCA => self.dex(),
        0xCB => self.immediate_cb("AXS", &mut Self::axs),
        0xCC => self.absolute_cb("CPY", &mut Self::cpy),
        0xCD => self.absolute_cb("CMP", &mut Self::cmp),
        0xCE => self.dec_abs(),
        0xCF => self.dcp_absolute(),
        0xD0 => self.bne(),
        0xD1 => self.indexed_y_cb("CMP", &mut Self::cmp),
        0xD2 => self.kil(),
        0xD3 => self.dcp_indexed_y(),
        0xD4 => self.zp_reg_cb("DOP", self.x_register.get(), &mut Self::dop),
        0xD5 => self.zp_reg_cb("CMP", self.x_register.get(), &mut Self::cmp),
        0xD6 => self.dec_zp_reg(),
        0xD7 => self.dcp_zp_reg(),
        0xD8 => self.cld(),
        0xD9 => self.absolute_y_cb("CMP", &mut Self::cmp),
        0xDB => self.dcp_abs_y(),
        0xDA => self.nop(),
        0xDC => self.absolute_x_cb("TOP", &mut Self::top),
        0xDD => self.absolute_x_cb("CMP", &mut Self::cmp),
        0xDE => self.dec_abs_x(),
        0xDF => self.dcp_abs_x(),
        0xE0 => self.immediate_cb("CPX", &mut Self::cpx),
        0xE1 => self.indexed_x_cb("SBC", &mut Self::sbc),
        0xE2 => self.immediate_cb("DOP", &mut Self::dop),
        0xE3 => self.indexed_x_cb("ISC", &mut Self::isc),
        0xE4 => self.zero_page_cb("CPX", &mut Self::cpx),
        0xE5 => self.zero_page_cb("SBC", &mut Self::sbc),
        0xE6 => self.inc_zp(),
        0xE7 => self.zero_page_cb("ISC", &mut Self::isc),
        0xE8 => self.inx(),
        0xE9 => self.immediate_cb("SBC", &mut Self::sbc),
        0xEA => self.nop(),
        0xEB => self.immediate_cb("SBC", &mut Self::sbc),
        0xEC => self.absolute_cb("CPX", &mut Self::cpx),
        0xED => self.absolute_cb("SBC", &mut Self::sbc),
        0xEE => self.inc_abs(),
        0xEF => self.absolute_cb("ISC", &mut Self::isc),
        0xF0 => self.beq(),
        0xF1 => self.indexed_y_cb("SBC", &mut Self::sbc),
        0xF2 => self.kil(),
        0xF3 => self.indexed_y_cb("ISC", &mut Self::isc),
        0xF4 => self.zp_reg_cb("DOP", self.x_register.get(), &mut Self::dop),
        0xF5 => self.zp_reg_cb("SBC", self.x_register.get(), &mut Self::sbc),
        0xF6 => self.inc_zp_reg(),
        0xF7 => self.zp_reg_cb("ISC", self.x_register.get(), &mut Self::isc),
        0xF8 => self.sed(),
        0xF9 => self.absolute_y_cb("SBC", &mut Self::sbc),
        0xFA => self.nop(),
        0xFB => self.absolute_y_cb("ISC", &mut Self::isc),
        0xFC => self.absolute_x_cb("TOP", &mut Self::top),
        0xFD => self.absolute_x_cb("SBC", &mut Self::sbc),
        0xFE => self.inc_abs_x(),
        0xFF => self.absolute_x_cb("ISC", &mut Self::isc),
      }
    }
  }

  /*
  ============================================================================================
                                  Generic operations
  ============================================================================================
  */

  /// Immediate addressing mode. Costs one cycle.
  fn immediate(&mut self, name: &str) -> u8 {
    let op = self.get_single_operand();
    debug!("{} immediate called with operand:0x{:X}", name, op);
    op
  }

  /// Callback version of immediate addressing mode.
  fn immediate_cb<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let op = self.immediate(name);
    cb(self, op);
  }

  /// Zero page addressing mode. Costs two cycles.
  fn zero_page(&mut self, name: &str) -> (u8, u8) {
    let index = self.get_single_operand();
    debug!("{} zero page called with index: 0x{:X}", name, index);
    (index, self.get_zero_page(index))
  }

  /// Zero page addressing mode. Costs one cycle
  fn zero_page_index(&mut self, name: &str) -> u8 {
    let index = self.get_single_operand();
    debug!("{} zero page called with index: 0x{:X}", name, index);
    index
  }

  /// Callback version of zero page addressing mode.
  fn zero_page_cb<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let (_, value) = self.zero_page(name);
    cb(self, value);
  }

  /// Zero page x or y addressing mode. Costs 3 cycles.
  fn zp_reg(&mut self, name: &str, reg_val: u8) -> (u8, u8) {
    let op = self.get_single_operand();
    debug!("{} zero page x called with operand: 0x{:X}", name, op);
    // waste a cycle
    self.get_zero_page(op);
    let index = op.wrapping_add(reg_val);
    (index, self.get_zero_page(index))
  }

  /// Zero page x or y addressing mode. Costs 2 cycles.
  fn zp_reg_index(&mut self, name: &str, reg_val: u8) -> u8 {
    let op = self.get_single_operand();
    debug!("{} zero page x called with operand: 0x{:X}", name, op);
    // waste a cycle
    self.get_zero_page(op);
    op.wrapping_add(reg_val)
  }

  /// Callback version of zero page register addressing modes (x or y)
  fn zp_reg_cb<F: FnMut(&mut Self, u8)>(&mut self, name: &str, reg_val: u8, cb: &mut F) {
    let (_, value) = self.zp_reg(name, reg_val);
    cb(self, value);
  }

  /// Absolute addressing mode. Costs 3 cycles.
  fn absolute(&mut self, name: &str) -> (u16, u8) {
    let ops = self.get_two_operands();
    let index = u16::from_le_bytes(ops);
    debug!("{} absolute called with index: 0x{:X}", name, index);
    (index, self.get_u16(index))
  }

  /// Absolute addressing mode. Costs 2 cycles
  fn absolute_index(&mut self, name: &str) -> u16 {
    let ops = self.get_two_operands();
    let index = u16::from_le_bytes(ops);
    debug!("{} absolute called with index: 0x{:X}", name, index);
    index
  }

  /// Callback version of absolute addressing mode.
  fn absolute_cb<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let (_, value) = self.absolute(name);
    cb(self, value);
  }

  /// Absolute x or y addressing mode. Costs at least 3 cycles. Can add a cycle
  /// if a page boundary is crossed.
  fn absolute_reg(&mut self, name: &str, reg: u8) -> (u16, u8) {
    let ops = self.get_two_operands();
    let index = u16::from_le_bytes(ops);
    debug!("{} absolute reg called with index: 0x{:X}", name, index);
    let total = index.wrapping_add(reg as u16);
    self.test_for_overflow(ops[1], reg);
    (index, self.get_u16(total))
  }

  /// Callback version of Absolute X addressing mode.
  fn absolute_x_cb<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let (_, value) = self.absolute_reg(name, self.x_register.get());
    cb(self, value);
  }

  /// Callback version of Absolute Y addressing mode.
  fn absolute_y_cb<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let (_, value) = self.absolute_reg(name, self.y_register.get());
    cb(self, value);
  }

  /// AKA Indexed indirect AKA pre-indexed. Costs 4 cycles
  fn indexed_x(&mut self, name: &str) -> (u16, u8) {
    let op = self.get_single_operand();
    debug!("{} indexed x called with operand: 0x{:X}", name, op);
    let modified_op = op.wrapping_add(self.x_register.get());
    let lo = self.get_zero_page(modified_op);
    let hi = self.get_zero_page(modified_op.wrapping_add(1));
    let index = u16::from_le_bytes([lo, hi]);
    (index, self.get_u16(index))
  }

  /// Callback version of indexed x addressing mode.
  fn indexed_x_cb<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let (_, value) = self.indexed_x(name);
    cb(self, value);
  }

  /// AKA Indirect indexed AKA post-indexed. Costs 4 cycles
  fn indexed_y(&mut self, name: &str) -> (u16, u8) {
    let op = self.get_single_operand();
    debug!("{} indexed y called with operand: 0x{:X}", name, op);
    let y_val = self.y_register.get();
    let lo = self.get_zero_page(op);
    let hi = self.get_zero_page(op.wrapping_add(1));
    let index = u16::from_le_bytes([lo, hi]);
    self.test_for_overflow(hi, y_val);
    (index, self.memory.get_u16(index))
  }

  /// Callback version of indexed y addressing mode.
  fn indexed_y_cb<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let (_, value) = self.indexed_y(name);
    cb(self, value);
  }

  /// Generic handler for branching. Branching costs an extra machine cycle if
  /// the branch is taken, and additionally if the branch adjustment crosses
  /// a memory page boundary an additional machine cycle is required.
  ///
  /// If the operand is greater than 0x7F we assume it is negative and handle
  /// it as two's complement.
  fn branch(&mut self, condition: bool, op: u8) {
    if condition {
      let overflow = match op > 0x7F {
        // Funky syntax is two's complement. Cannot have negative unsigned.
        true => self.program_counter.decrease(!op + 1),
        false => self.program_counter.increase(op),
      };
      if overflow {
        // Page overflow costs a cycle
        self.sync();
      }
      // Branch taken costs a cycle
      self.sync();
      debug!(
        "Branch taken. Execution resuming at {}",
        self.program_counter.get()
      );
    }
  }

  /// Tests if a value meets specific criteria and sets bits as appropriate.
  ///
  /// reg_value can be any of the 3 generic registers. If the reg_value is
  /// greater than or equal to the test_value, the carry is set.
  fn generic_compare(&mut self, test_value: u8, reg_value: u8) {
    trace!("Comparing values");
    let (result, carry) = reg_value.overflowing_sub(test_value);
    if result == 0 {
      self.status_register.set_flag(StatusBit::Zero);
    }
    // Carry is true only when the test value is greater than the register value.
    if !carry {
      self.status_register.set_flag(StatusBit::Carry);
    }
    // Check if bit 7 is set
    if (result & 0x80) > 0 {
      self.status_register.set_flag(StatusBit::Negative);
    }
  }

  /// Generic register operation, such as transfer accumulator to x register.
  ///
  /// These are one byte instructions so we need to wait for a machine cycle,
  /// We always check the same flags so we do so generically.
  fn register_operation(&mut self, value: u8, message: &str) {
    debug!("{} called reg operation with value: {}", message, value);
    self.status_register.handle_n_flag(value, message);
    self.status_register.handle_z_flag(value, message);
    self.sync();
  }

  /// Rotates bits to the right.
  ///
  /// The carry is shifted into bit 7, and bit 0 is shifted to the carry.
  fn rotate_right(&mut self, value: u8) -> u8 {
    debug!("Rotating bits to the right");
    let mut result = value.wrapping_shr(1);
    if self.status_register.is_flag_set(StatusBit::Carry) {
      result |= 0x80;
    }
    match value & 0x1 == 1 {
      true => self.status_register.set_flag(StatusBit::Carry),
      false => self.status_register.clear_flag(StatusBit::Carry),
    }
    result
  }

  /// Shifts bits to the right.
  ///
  /// Zero is shifted into bit 0 and bit 7 is shifted into the carry
  fn shift_right(&mut self, value: u8) -> u8 {
    debug!("Shifting bits to the right");
    let result = value.wrapping_shr(1);
    match value & 0x1 == 1 {
      true => self.status_register.set_flag(StatusBit::Carry),
      false => self.status_register.clear_flag(StatusBit::Carry),
    }
    result
  }

  /// Rotates bits to the left.
  ///
  /// The carry is shifted into bit 0 and bit 7 is shifted into the carry
  fn rotate_left(&mut self, value: u8) -> u8 {
    debug!("Rotating bits to the left");
    let mut result = value.wrapping_shl(1);
    if self.status_register.is_flag_set(StatusBit::Carry) {
      result |= 0x1;
    }
    match value & 0x80 == 0x80 {
      true => self.status_register.set_flag(StatusBit::Carry),
      false => self.status_register.clear_flag(StatusBit::Carry),
    }
    result
  }

  /// Shifts bits to the left
  ///
  /// 0 is shifted into bit 0 and bit 7 is shifted into the carry
  fn shift_left(&mut self, value: u8) -> u8 {
    debug!("Shift bits to the left");
    let result = value.wrapping_shl(1);
    match value & 0x80 == 0x80 {
      true => self.status_register.set_flag(StatusBit::Carry),
      false => self.status_register.clear_flag(StatusBit::Carry),
    }
    result
  }

  /*
  ============================================================================================
                                  Interrupts
  ============================================================================================
  */

  /// Services any type of interrupt.
  ///
  /// Interrupts require 7 machine cycles.
  /// Interrupts push the program counter to the stack, low byte first.
  /// Interrupts push the status register to the stack.
  /// Each interrupt type has an address it looks to in order to process instructions, which
  /// informs how to handle the IRQ.
  fn interrupt(&mut self, low_vec: u16, hi_vec: u16) -> u16 {
    trace!("Starting interrupt request");
    self.status_register.set_flag(StatusBit::Interrupt);
    self.internal_operations();
    let ops = self.program_counter.get().to_le_bytes();
    self.push_to_stack(ops[0]);
    self.push_to_stack(ops[1]);
    self.push_to_stack(self.status_register.get_register());
    let lo = self.get_u16(low_vec);
    let hi = self.get_u16(hi_vec);
    trace!("Interrupt start up complete. Starting interrupt execution");
    u16::from_le_bytes([lo, hi])
  }

  /// Returns from an interrupt.
  ///
  /// Restores the cpu back to the state it was before the interrupt transpired.
  /// Takes 6 cycles to execute.
  fn return_from_interrupt(&mut self) {
    trace!("Starting to return from interrupt");
    let status_reg = self.pop_from_stack();
    self.status_register.set(status_reg);
    let hi_pc = self.pop_from_stack();
    let lo_pc = self.pop_from_stack();
    self
      .program_counter
      .jump(u16::from_le_bytes([lo_pc, hi_pc]));
    self.status_register.clear_flag(StatusBit::Interrupt);
    self.status_register.clear_flag(StatusBit::Break);
    trace!("Interrupt return complete. Resuming normal operation");
  }

  /// Unspecified thing that delays execution by two cycles. Used for interrupts.
  fn internal_operations(&mut self) {
    self.sync();
    self.sync();
  }

  /// Resets the system. Some data will be left over after depending on where
  /// the program was in the execution cycle
  fn reset_interrupt(&mut self) {
    let index = self.interrupt(0xFFFC, 0xFFFD);
    self.program_counter.jump(index);
    debug!("Reset interrupt called");
    self.reset();
  }

  /// Calls a non-maskable interrupt.
  fn nmi_interrupt(&mut self) {
    let index = self.interrupt(0xFFFA, 0xFFFB);
    debug!("NMI interrupt called");
    self.program_counter.jump(index);
  }

  /// Calls a regular interrupt.
  fn irq_interrupt(&mut self) {
    let index = self.interrupt(0xFFFE, 0xFFFF);
    debug!("IRQ interrupt called");
    self.program_counter.jump(index);
  }

  /*
  ============================================================================================
                                  Opcodes
  ============================================================================================
  */

  /// Illegal opcode.
  /// Two opcode values reference this, both are immediate mode.
  ///
  /// Affects flags N Z C. Carry is set if result is negative
  pub fn aac(&mut self, value: u8) {
    let message = "AAC";
    warn!("{} called. Something might be borked.", message);
    let result = value & self.accumulator.get();
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_z_flag(result, message);
    self.status_register.handle_c_flag(
      message,
      self.status_register.is_flag_set(StatusBit::Negative),
    );
  }

  /// Illegal opcode.
  /// And x register with accumulator and store result in memory.
  /// Four possible codes, zero page, zero page y, indirect x, and absolute
  ///
  /// Affects flags N, Z
  pub fn aax(&mut self, index: u16) {
    let message = "AAX";
    warn!("{} called. Something might be borked.", message);
    let result = self.x_register.get() & self.accumulator.get();
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_z_flag(result, message);
    self.set_u16(index, result);
  }

  /// Performs AAX in zero page addressing mode
  pub fn aax_zero_page(&mut self) {
    let index = self.zero_page_index("AAX");
    self.aax(index as u16);
  }

  /// Performs AAX in zero page x addressing mode
  pub fn aax_zero_page_y(&mut self) {
    let index = self.zp_reg_index("AAX", self.y_register.get());
    self.aax(index as u16);
  }

  /// Performs AAX in indexed x addressing mode
  pub fn aax_indirect_x(&mut self) {
    let (index, _) = self.indexed_x("AAX");
    self.aax(index);
  }

  /// Performs AAX in absolute addressing mode
  pub fn aax_absolute(&mut self) {
    let index = self.absolute_index("AAX");
    self.aax(index);
  }

  // TODO: Add BCD
  /// ADd with Carry
  ///
  /// Adds the value given to the accumulator including the carry.
  /// Uses BCD format if decimal flag is set.
  ///
  /// Affects flags N V Z C
  pub fn adc(&mut self, value: u8) {
    let message = "ADC";
    debug!("{} called with value: 0x{:X}", message, value);
    let modifier = match self.status_register.is_flag_set(StatusBit::Carry) {
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

  /// AND accumulator
  ///
  /// Bitwise AND operation performed against the accumulator
  ///
  /// Affects flags N Z
  pub fn and(&mut self, value: u8) {
    let message = "AND";
    debug!("{} called with value: 0x{:X}", message, value);
    let result = self.accumulator.get() & value;
    self.accumulator.set(result);
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_z_flag(result, message);
  }

  /// Illegal opcode.
  /// And operand with accumulator, then rotate one bit right, then
  /// check bits 5 and 6.
  ///
  /// Affects flags N V Z C
  pub fn arr(&mut self, value: u8) {
    let message = "ARR";
    warn!("{} called. Something might be borked.", message);
    let result = self.accumulator.get() & value;
    let result = self.rotate_right(result);
    self.accumulator.set(result);
    let b5 = (result & 0x20) >> 5;
    let b6 = (result & 0x40) >> 6;
    if b5 == 1 && b6 == 1 {
      self.status_register.set_flag(StatusBit::Carry);
      self.status_register.clear_flag(StatusBit::Overflow);
    } else if b5 == 0 && b6 == 0 {
      self.status_register.clear_flag(StatusBit::Carry);
      self.status_register.clear_flag(StatusBit::Overflow);
    } else if b5 == 1 && b6 == 0 {
      self.status_register.set_flag(StatusBit::Overflow);
      self.status_register.clear_flag(StatusBit::Carry);
    } else {
      self.status_register.set_flag(StatusBit::Overflow);
      self.status_register.set_flag(StatusBit::Carry);
    }
    self.status_register.handle_z_flag(result, message);
    self.status_register.handle_n_flag(result, message);
  }

  /// Arithmetic Shift Left
  ///
  /// Shifts all bits left one position for the applied location
  ///
  /// Affects flags N Z C
  fn asl(&mut self, value: u8) -> u8 {
    let message = "ASL";
    debug!("{} called with value: 0x{:X}", message, value);
    let (result, carry) = value.overflowing_shl(1);
    // extra cycle for modification
    self.sync();
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_z_flag(result, message);
    self.status_register.handle_c_flag(message, carry);
    result
  }

  /// Performs ASL in accumulator addressing mode
  pub fn asl_accumulator(&mut self) {
    let result = self.asl(self.accumulator.get());
    trace!("ASL accumulator called");
    self.accumulator.set(result);
  }

  /// Performs ASL in zero page addressing mode
  pub fn asl_zero_page(&mut self) {
    let (index, value) = self.zero_page("ASL");
    let result = self.asl(value);
    self.set_zero_page(index, result);
  }

  /// Performs ASL in zero page x addressing mode
  pub fn asl_zero_page_x(&mut self) {
    let (index, value) = self.zp_reg("ASL", self.x_register.get());
    let result = self.asl(value);
    self.set_zero_page(index, result);
  }

  /// Performs ASL in absolute addressing mode
  pub fn asl_absolute(&mut self) {
    let (index, value) = self.absolute("ASL");
    let result = self.asl(value);
    self.set_u16(index, result);
  }

  /// Performs ASL in absolute x addressing mode
  pub fn asl_absolute_x(&mut self) {
    let (index, value) = self.absolute_reg("ASL", self.x_register.get());
    let result = self.asl(value);
    self.set_u16(index, result);
    // extra cycle. do not know from where
    self.sync();
  }

  /// Illegal opcode.
  /// AND byte with accumulator, then shift right one bit
  /// in accumulator
  ///
  /// Affects flags N Z C
  pub fn asr(&mut self, value: u8) {
    let message = "ASR";
    warn!("{} called. Something might be borked.", message);
    let result = self.accumulator.get() & value;
    let result = self.shift_right(result);
    self.accumulator.set(result);
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_z_flag(result, message);
  }

  /// Illegal opcode.
  /// AND byte with accumulator, then transfer accumulator to X register.
  ///
  /// Affects flags N Z
  pub fn atx(&mut self, value: u8) {
    let message = "ATX";
    warn!("{} called. Something might be borked.", message);
    let result = self.accumulator.get() & value;
    self.accumulator.set(result);
    self.x_register.set(result);
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_z_flag(result, message);
  }

  /// Illegal opcode.
  /// AND X register with accumulator then AND result with 7
  /// and store in memory
  /// Affects no flags
  pub fn axa(&mut self, index: u16) {
    warn!("AXA called. Something might be borked.");
    let result = self.accumulator.get() & self.x_register.get();
    let result = result & 7;
    self.set_u16(index, result);
  }

  /// Performs AXA in absolute y addressing mode
  pub fn axa_absolute_y(&mut self) {
    let ops = self.get_two_operands();
    let index = u16::from_le_bytes(ops);
    let reg = self.y_register.get();
    let index = index.wrapping_add(reg as u16);
    self.test_for_overflow(ops[1], reg);
    self.axa(index);
  }

  /// Performs AXA in indexed x addressing mode
  pub fn axa_indirect(&mut self) {
    let op = self.get_single_operand();
    let modified_op = op.wrapping_add(self.x_register.get());
    let lo = self.get_zero_page(modified_op);
    let hi = self.get_zero_page(modified_op.wrapping_add(1));
    let index = u16::from_le_bytes([lo, hi]);
    self.axa(index);
  }

  /// Illegal opcode.
  /// AND x register with accumulator and store result in x register.
  /// Then subtract byte from x register (no borrow)
  ///
  /// Affects flags N Z C
  pub fn axs(&mut self, value: u8) {
    let message = "AXS";
    warn!("{} called. Something might be borked.", message);
    let result = self.x_register.get() & self.accumulator.get();
    let (result, carry) = result.overflowing_sub(value);
    self.x_register.set(result);
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_z_flag(result, message);
    self.status_register.handle_c_flag(message, carry);
  }

  /// Tests a value and sets flags accordingly.
  ///
  /// Zero is set by looking at the result of the value AND the accumulator.
  /// N & V are set by bits 7 & 6 of the value respectively.
  ///
  /// Affects flags N V Z
  fn bit(&mut self, value_to_test: u8) {
    debug!("BIT called checking {}", value_to_test);
    let n_result = self.accumulator.get() & value_to_test;
    self.status_register.handle_n_flag(value_to_test, "BIT");
    self.status_register.handle_z_flag(n_result, "BIT");
    if (value_to_test & 0x40) >> 6 == 1 {
      self.status_register.set_flag(StatusBit::Overflow);
    } else {
      self.status_register.clear_flag(StatusBit::Overflow);
    }
  }

  /// Branch on PLus.
  ///
  /// Checks the negative bit and branches if it is clear.
  pub fn bpl(&mut self) {
    debug!("BPL called");
    let op = self.get_single_operand();
    self.branch(!self.status_register.is_flag_set(StatusBit::Negative), op);
  }

  /// Branch on MInus
  ///
  /// Checks the negative bit and branches if it is set.
  pub fn bmi(&mut self) {
    debug!("BMI called");
    let op = self.get_single_operand();
    self.branch(self.status_register.is_flag_set(StatusBit::Negative), op);
  }

  /// Branch on oVerflow Clear
  ///
  /// Checks the overflow bit and branches if it is clear.
  pub fn bvc(&mut self) {
    debug!("BVC called");
    let op = self.get_single_operand();
    self.branch(!self.status_register.is_flag_set(StatusBit::Overflow), op);
  }

  /// Branch on oVerflow Set
  ///
  /// Checks the overflow bit and branches if it is set.
  pub fn bvs(&mut self) {
    debug!("BVS called");
    let op = self.get_single_operand();
    self.branch(self.status_register.is_flag_set(StatusBit::Overflow), op);
  }

  /// Branch on Carry Clear
  ///
  /// Checks the carry bit and branches if it is clear.
  pub fn bcc(&mut self) {
    debug!("BCC called");
    let op = self.get_single_operand();
    self.branch(!self.status_register.is_flag_set(StatusBit::Carry), op);
  }

  /// Branch on Carry Set
  ///
  /// Checks the carry bit and branches if it is set.
  pub fn bcs(&mut self) {
    debug!("BSC called");
    let op = self.get_single_operand();
    self.branch(self.status_register.is_flag_set(StatusBit::Carry), op);
  }

  /// Branch on Not Equal
  ///
  /// Checks the zero bit and branches if it is clear.
  pub fn bne(&mut self) {
    debug!("BNE called");
    let op = self.get_single_operand();
    self.branch(!self.status_register.is_flag_set(StatusBit::Zero), op);
  }

  /// Branch on EQual
  ///
  /// Checks the zero bit and branches if it is set.
  pub fn beq(&mut self) {
    debug!("BEQ called");
    let op = self.get_single_operand();
    self.branch(self.status_register.is_flag_set(StatusBit::Zero), op);
  }

  /// BReaK
  ///
  /// Performs an irq interrupt. Software side so used for debugging.
  pub fn brk(&mut self) {
    warn!("BRK called. Are you debugging?");
    self.status_register.set_flag(StatusBit::Break);
    self.program_counter.increase(1);
    self.irq_interrupt();
  }

  /// CoMPare accumulator
  ///
  /// Checks a test value against the value in the accumulator and sets
  /// flags accordingly.
  pub fn cmp(&mut self, test_value: u8) {
    debug!("CMP called");
    self.generic_compare(test_value, self.accumulator.get());
  }

  /// ComPare X register
  ///
  /// Checks a test value against the value in the x register and sets
  /// flags accordingly.
  pub fn cpx(&mut self, test_value: u8) {
    debug!("CPX called");
    self.generic_compare(test_value, self.x_register.get());
  }

  /// ComPare Y register
  ///
  /// Checks a test value against the value in the y register and sets
  /// flags accordingly.
  pub fn cpy(&mut self, test_value: u8) {
    debug!("CPY called");
    self.generic_compare(test_value, self.y_register.get());
  }

  pub fn clc(&mut self) {
    self.status_register.clear_flag(StatusBit::Carry);
    // All cycles need 2 bytes
    self.sync();
  }

  pub fn cld(&mut self) {
    self.status_register.clear_flag(StatusBit::Decimal);
    // All cycles need 2 bytes
    self.sync();
  }

  pub fn cli(&mut self) {
    self.status_register.clear_flag(StatusBit::Interrupt);
    // All cycles need 2 bytes
    self.sync();
  }

  pub fn clv(&mut self) {
    self.status_register.clear_flag(StatusBit::Overflow);
    // All cycles need 2 bytes
    self.sync();
  }

  /// Illegal opcode.
  /// Subtract 1 from memory (without borrow)
  ///
  /// Affects flags C
  pub fn dcp(&mut self, index: u16, value: u8) {
    let message = "DCP";
    warn!("{} called. Something might be borked.", message);
    let (result, carry) = value.overflowing_sub(1);
    self.status_register.handle_c_flag(message, carry);
    self.set_u16(index, result);
  }

  pub fn dcp_zp(&mut self) {
    let (index, value) = self.zero_page("DCP");
    self.dcp(index as u16, value);
  }

  pub fn dcp_zp_reg(&mut self) {
    let (index, value) = self.zp_reg("DEC", self.x_register.get());
    self.dcp(index as u16, value);
  }

  pub fn dcp_absolute(&mut self) {
    let (index, value) = self.absolute("DEC");
    self.dcp(index, value);
  }

  pub fn dcp_abs_x(&mut self) {
    let (index, value) = self.absolute_reg("DCP", self.x_register.get());
    self.dcp(index, value);
  }

  pub fn dcp_abs_y(&mut self) {
    let (index, value) = self.absolute_reg("DCP", self.y_register.get());
    self.dcp(index, value);
  }

  pub fn dcp_indexed_x(&mut self) {
    let (index, value) = self.indexed_x("DCP");
    self.dcp(index, value);
  }

  pub fn dcp_indexed_y(&mut self) {
    let (index, value) = self.indexed_y("DCP");
    self.dcp(index, value);
  }

  pub fn dec(&mut self, index: u16, value: u8) {
    let value = value.wrapping_sub(1);
    // extra cycle for modification
    self.sync();
    trace!("DEC called index: {}, value: {}", index, value);
    self.set_u16(index, value);
    self.status_register.handle_n_flag(value, "DEC");
    self.status_register.handle_z_flag(value, "DEC");
  }

  pub fn dec_zp(&mut self) {
    let (index, value) = self.zero_page("DEC");
    self.dec(index as u16, value);
  }

  pub fn dec_zp_reg(&mut self) {
    let (index, value) = self.zp_reg("DEC", self.x_register.get());
    self.dec(index as u16, value);
  }

  pub fn dec_abs(&mut self) {
    let (index, value) = self.absolute("DEC");
    self.dec(index as u16, value);
  }

  pub fn dec_abs_x(&mut self) {
    let (index, value) = self.absolute_reg("DEC", self.x_register.get());
    self.dec(index, value);
    // extra cycle. do not know why
    self.sync();
  }

  /// Illegal opcode
  /// Nop.
  pub fn dop(&mut self, _: u8) {
    warn!("DOP called. Something might be borked");
    self.nop();
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

  pub fn inc(&mut self, index: u16, value: u8) {
    let value = value.wrapping_add(1);
    // extra cycle for modification
    self.sync();
    trace!("INC called index: {}, value: {}", index, value);
    self.set_u16(index, value);
    self.status_register.handle_n_flag(value, "INC");
    self.status_register.handle_z_flag(value, "INC");
  }

  pub fn inc_zp(&mut self) {
    let (index, value) = self.zero_page("INC");
    self.inc(index as u16, value);
  }

  pub fn inc_zp_reg(&mut self) {
    let (index, value) = self.zp_reg("INC", self.x_register.get());
    self.inc(index as u16, value);
  }

  pub fn inc_abs(&mut self) {
    let (index, value) = self.absolute("INC");
    self.inc(index as u16, value);
  }

  pub fn inc_abs_x(&mut self) {
    let (index, value) = self.absolute_reg("INC", self.x_register.get());
    self.inc(index as u16, value);
    // extra cycle. do not know why
    self.sync();
  }

  /// Illegal opcode.
  /// Increase memory by one, then subtract memory from accu-mulator (with borrow)
  ///
  /// Affects flags N V Z C
  pub fn isc(&mut self, value: u8) {
    let message = "ISC";
    warn!("{} called. Something might be borked", message);
    let result = value.wrapping_add(1);
    let (result, carry) = self.accumulator.get().overflowing_sub(result);
    self.accumulator.set(result);
    self.status_register.handle_c_flag(message, carry);
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_v_flag(result, message, carry);
    self.status_register.handle_z_flag(result, message);
  }

  /// Illegal opcode.
  /// Locks the system, so we simulate by panic
  pub fn kil(&self) {
    panic!("KIL called. CPU is locked.");
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
    let hi = self.get_u16(u16::from_le_bytes(ops));
    let lo = self.get_u16(u16::from_le_bytes([low_test, ops[1]]));
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
    // extra cycle needed due the return address
    self.sync();
    self.program_counter.jump(index);
  }

  /// Illegal opcode.
  /// AND memory with stack pointer, transfer result to accumulator,
  /// X register and stack pointer.
  ///
  /// Affects flags N Z
  pub fn lar(&mut self, value: u8) {
    let message = "LAR";
    warn!("{} called. Something might be borked.", message);
    let result = value & self.memory.get_stack_pointer().get();
    self.accumulator.set(result);
    self.x_register.set(result);
    self.memory.set_stack_pointer(result);
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_z_flag(result, message);
  }

  /// Illegal opcode.
  /// Load accumulator and X register with memory.
  ///
  /// Affects flags N Z
  pub fn lax(&mut self, value: u8) {
    let message = "LAX";
    warn!("{} called. Something might be borked.", message);
    self.accumulator.set(value);
    self.x_register.set(value);
    self.status_register.handle_n_flag(value, message);
    self.status_register.handle_z_flag(value, message);
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
    // extra cycle for modification
    self.sync();
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
    self.set_zero_page(index, result);
  }

  pub fn lsr_zero_page_x(&mut self) {
    let (index, value) = self.zp_reg("LSR", self.x_register.get());
    let result = self.lsr(value);
    self.set_zero_page(index, result);
  }

  pub fn lsr_absolute(&mut self) {
    let (index, value) = self.absolute("LSR");
    let result = self.lsr(value);
    self.set_u16(index, result);
  }

  pub fn lsr_absolute_x(&mut self) {
    let (index, value) = self.absolute_reg("LSR", self.x_register.get());
    let result = self.lsr(value);
    self.set_u16(index, result);
  }

  pub fn nop(&mut self) {
    // Extra cycle as all instruction require two bytes.
    self.sync();
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
    self.x_register.set(self.x_register.get());
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

  /// Illegal opcode.
  /// Rotate one bit left in memory, then AND accumulator with memory.
  ///
  /// Affects flags N Z C
  pub fn rla(&mut self, value: u8) {
    let message = "RLA";
    warn!("{} called. Something might be borked", message);
    let result = self.rotate_left(value);
    let result = self.accumulator.get() & result;
    self.accumulator.set(result);
    self.status_register.handle_z_flag(result, message);
    self.status_register.handle_n_flag(result, message);
  }

  fn rol(&mut self, value: u8) -> u8 {
    trace!("ROL called with value: {}", value);
    let result = self.rotate_left(value);
    // extra cycle for modification
    self.sync();
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
    self.set_zero_page(index, result);
  }

  pub fn rol_zero_page_x(&mut self) {
    let (index, value) = self.zp_reg("ROL", self.x_register.get());
    let result = self.rol(value);
    self.set_zero_page(index, result);
  }

  pub fn rol_absolute(&mut self) {
    let (index, value) = self.absolute("ROL");
    let result = self.rol(value);
    self.set_u16(index, result);
  }

  pub fn rol_absolute_x(&mut self) {
    let (index, value) = self.absolute_reg("ROL", self.x_register.get());
    let result = self.rol(value);
    self.set_u16(index, result);
  }

  fn ror(&mut self, value: u8) -> u8 {
    trace!("ROR called with value: {}", value);
    let result = self.rotate_right(value);
    // extra cycle for modification
    self.sync();
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
    self.set_zero_page(index, result);
  }

  pub fn ror_zero_page_x(&mut self) {
    let (index, value) = self.zp_reg("ROR", self.x_register.get());
    let result = self.ror(value);
    self.set_zero_page(index, result);
  }

  pub fn ror_absolute(&mut self) {
    let (index, value) = self.absolute("ROR");
    let result = self.ror(value);
    self.set_u16(index, result);
  }

  pub fn ror_absolute_x(&mut self) {
    let (index, value) = self.absolute_reg("ROR", self.x_register.get());
    let result = self.ror(value);
    self.set_u16(index, result);
  }

  /// Illegal opcode.
  /// Rotate one bit right in memory, then add memory to accumulator (with
  /// carry).

  /// Affects flags N V Z C
  pub fn rra(&mut self, value: u8) {
    let message = "RRA";
    warn!("{} called. Something might be borked", message);
    let result = self.rotate_right(value);
    let modifier = match self.status_register.is_flag_set(StatusBit::Carry) {
      true => 1,
      false => 0,
    };
    let (result, c1) = result.overflowing_add(modifier);
    let (result, c2) = self.accumulator.get().overflowing_add(result);
    let carry = c1 || c2;
    self.accumulator.set(result);
    self.status_register.handle_c_flag(message, carry);
    self.status_register.handle_v_flag(result, message, carry);
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_z_flag(result, message);
  }

  /// Return from interrupt
  pub fn rti(&mut self) {
    self.return_from_interrupt();
  }

  pub fn rts(&mut self) {
    let lo = self.pop_from_stack();
    let hi = self.pop_from_stack();
    let index = u16::from_le_bytes([lo, hi]) + 1;
    // extra cycle to increment the index
    self.sync();
    self.program_counter.jump(index);
    // one byte extra cycle
    self.sync();
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
    self.status_register.set_flag(StatusBit::Carry);
    // All ops require two bytes
    self.sync();
  }

  pub fn sed(&mut self) {
    self.status_register.set_flag(StatusBit::Decimal);
    // All ops require two bytes
    self.sync();
  }

  pub fn sei(&mut self) {
    self.status_register.set_flag(StatusBit::Interrupt);
    // All ops require two bytes
    self.sync();
  }

  /// Illegal opcode.
  /// Shift left one bit in memory, then OR accumulator with memory
  ///
  /// Affects flags N Z C
  pub fn slo(&mut self, value: u8) {
    let message = "SLO";
    warn!("{} called. Something might be borked", message);
    let result = self.shift_left(value);
    let result = self.accumulator.get() | result;
    self.accumulator.set(result);
    self.status_register.handle_z_flag(result, message);
    self.status_register.handle_n_flag(result, message);
  }

  /// Illegal opcode.
  /// Shift right one bit in memory, then EOR accumulator with memory
  ///
  /// Affects flags N Z C
  pub fn sre(&mut self, value: u8) {
    let message = "SRE";
    warn!("{} called. Something might be borked", message);
    let result = self.shift_right(value);
    let result = self.accumulator.get() ^ result;
    self.accumulator.set(result);
    self.status_register.handle_z_flag(result, message);
    self.status_register.handle_n_flag(result, message);
  }

  pub fn sta_zero_page(&mut self) {
    let index = self.zero_page_index("STA");
    self.set_zero_page(index, self.accumulator.get());
  }

  pub fn sta_zero_page_x(&mut self) {
    let index = self.zp_reg_index("STA", self.x_register.get());
    self.set_zero_page(index, self.accumulator.get());
  }

  pub fn sta_absolute(&mut self) {
    let index = self.absolute_index("STA");
    self.set_u16(index, self.accumulator.get());
  }

  pub fn sta_absolute_x(&mut self) {
    let (index, _) = self.absolute_reg("STA", self.x_register.get());
    self.set_u16(index, self.accumulator.get());
  }

  pub fn sta_absolute_y(&mut self) {
    let (index, _) = self.absolute_reg("STA", self.y_register.get());
    self.set_u16(index, self.accumulator.get());
  }

  pub fn sta_indexed_x(&mut self) {
    let (index, _) = self.indexed_x("STA");
    self.set_u16(index, self.accumulator.get());
  }

  pub fn sta_indexed_y(&mut self) {
    let (index, _) = self.indexed_y("STA");
    self.set_u16(index, self.accumulator.get());
  }

  /// Illegal opcode.
  /// AND X register with the high byte of the target address of the
  /// argument + 1. Result stored in memory.
  pub fn sxa(&mut self) {
    warn!("SXA called. Something might be borked");
    let ops = self.get_two_operands();
    let result = (self.x_register.get() & ops[1]).wrapping_add(1);
    let index = u16::from_le_bytes(ops);
    self.set_u16(index, result);
  }

  /// Illegal opcode.
  /// AND Y register with the high byte of the target address of the
  /// argument + 1. Result stored in memory.
  pub fn sya(&mut self) {
    warn!("SYA called. Something might be borked");
    let ops = self.get_two_operands();
    let result = (self.y_register.get() & ops[1]).wrapping_add(1);
    let index = u16::from_le_bytes(ops);
    self.set_u16(index, result);
  }

  /// Illegal opcode
  /// Nop.
  pub fn top(&mut self, _: u8) {
    warn!("TOP called. Something might be borked");
    self.nop();
  }

  pub fn txs(&mut self) {
    self.memory.set_stack_pointer(self.x_register.get());
    // extra instruction byte always happens
    self.sync();
  }

  pub fn tsx(&mut self) {
    self.x_register.set(self.memory.get_stack_pointer().get());
    // extra instruction byte always happens
    self.sync();
  }

  pub fn pha(&mut self) {
    self.push_to_stack(self.accumulator.get());
    // extra instruction byte always happens
    self.sync();
  }

  pub fn pla(&mut self) {
    let stack_value = self.pop_from_stack();
    self.accumulator.set(stack_value);
    // extra instruction byte always happens
    self.sync();
  }

  pub fn php(&mut self) {
    self.push_to_stack(self.status_register.get_register());
    // extra instruction byte always happens
    self.sync();
  }

  pub fn plp(&mut self) {
    let stack = self.pop_from_stack();
    self.status_register.set(stack);
    // extra instruction byte always happens
    self.sync();
  }

  pub fn stx_zero_page(&mut self) {
    let index = self.zero_page_index("STX");
    self.set_zero_page(index, self.x_register.get());
  }

  pub fn stx_zero_page_y(&mut self) {
    let index = self.zp_reg_index("STX", self.y_register.get());
    self.set_zero_page(index, self.x_register.get());
  }

  pub fn stx_absolute(&mut self) {
    let index = self.absolute_index("STX");
    self.set_u16(index, self.x_register.get());
  }

  pub fn sty_zero_page(&mut self) {
    let index = self.zero_page_index("STY");
    self.set_zero_page(index, self.y_register.get());
  }

  pub fn sty_zero_page_x(&mut self) {
    let index = self.zp_reg_index("STY", self.x_register.get());
    self.set_zero_page(index, self.y_register.get());
  }

  pub fn sty_absolute(&mut self) {
    let index = self.absolute_index("STY");
    self.set_u16(index, self.y_register.get());
  }

  /// Illegal opcode.
  /// Panics as there is no definition of how this behaves.
  pub fn xaa(&mut self) {
    panic!("XAA called. Undefined and unknown behavior");
  }

  /// Illegal opcode.
  /// AND X register with accumulator and store result in stack pointer, then
  /// AND stack pointer with the high byte of the target address of the
  /// argument + 1. Store result in memory.
  ///
  /// Programmers note: WTF is this?!
  pub fn xas(&mut self) {
    let message = "XAS";
    warn!("{} called. Something might be borked", message);
    let result = self.x_register.get() & self.accumulator.get();
    self.memory.set_stack_pointer(result);
    let ops = self.get_two_operands();
    let result = (result & ops[1]) + 1;
    let index = u16::from_le_bytes(ops);
    self.set_u16(index, result);
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
