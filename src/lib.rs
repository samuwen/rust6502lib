mod memory;
mod registers;

use log::{debug, trace, warn};
use memory::Memory;
use registers::{GeneralRegister, ProgramCounter, StackPointer, StatusBit, StatusRegister};
use std::fmt::{Display, Formatter, Result};
use std::sync::mpsc::Receiver;

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
  clock_pin: Receiver<bool>,
}

impl CPU {
  /// Initializes a new CPU instance. Sets all values to their associated defaults.
  pub fn new(r: Receiver<bool>) -> CPU {
    debug!("Initializing CPU");
    CPU {
      program_counter: ProgramCounter::new(),
      accumulator: GeneralRegister::new(),
      x_register: GeneralRegister::new(),
      y_register: GeneralRegister::new(),
      status_register: StatusRegister::new(),
      memory: Memory::new(),
      clock_pin: r,
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
    // let b = self.clock_pin.recv().unwrap();
    while !self.clock_pin.try_recv().is_ok() {
      self.check_pins();
    }
    // self.check_pins();
    trace!("Starting machine cycle");
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
    debug!("Getting an operand with value: {:X}", op);
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
  pub fn run(&mut self, program: Vec<u8>, start: Option<u16>) -> bool {
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
    return false;
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
    // waste a cycle - bug in processor causes it to read the value
    self.get_zero_page(op);
    let index = op.wrapping_add(reg_val);
    (index, self.get_zero_page(index))
  }

  /// Zero page x or y addressing mode. Costs 2 cycles.
  fn zp_reg_index(&mut self, name: &str, reg_val: u8) -> u8 {
    let op = self.get_single_operand();
    debug!("{} zero page x called with operand: 0x{:X}", name, op);
    // waste a cycle - bug in processor causes it to read the value
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
    let index = index.wrapping_add(reg as u16);
    self.test_for_overflow(ops[1], reg);
    (index, self.get_u16(index))
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

  /// AKA Indexed indirect AKA pre-indexed. Costs 5 cycles
  fn indexed_x(&mut self, name: &str) -> (u16, u8) {
    let op = self.get_single_operand();
    debug!("{} indexed x called with operand: 0x{:X}", name, op);
    let modified_op = op.wrapping_add(self.x_register.get());
    let lo = self.get_zero_page(modified_op);
    let hi = self.get_zero_page(modified_op.wrapping_add(1));
    let index = u16::from_le_bytes([lo, hi]);
    // not sure where this extra cycle comes from.
    self.sync();
    (index, self.get_u16(index))
  }

  /// Callback version of indexed x addressing mode.
  fn indexed_x_cb<F: FnMut(&mut Self, u8)>(&mut self, name: &str, cb: &mut F) {
    let (_, value) = self.indexed_x(name);
    cb(self, value);
  }

  /// AKA Indirect indexed AKA post-indexed. Costs 5 cycles
  fn indexed_y(&mut self, name: &str) -> (u16, u8) {
    let op = self.get_single_operand();
    debug!("{} indexed y called with operand: 0x{:X}", name, op);
    let y_val = self.y_register.get();
    let lo = self.get_zero_page(op);
    let hi = self.get_zero_page(op.wrapping_add(1));
    let index = u16::from_le_bytes([lo, hi]);
    let index = index.wrapping_add(y_val as u16);
    self.test_for_overflow(hi, y_val);
    (index, self.get_u16(index))
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
        true => self.program_counter.decrease(!(op + 1)),
        false => self.program_counter.increase(op),
      };
      if overflow {
        // Page overflow costs a cycle
        self.sync();
      }
      // Branch taken costs a cycle
      self.sync();
      debug!(
        "Branch taken. Execution resuming at {:X}",
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
    debug!("{} called reg operation with value: {:X}", message, value);
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
    match (value & 0x80) == 0x80 {
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

  /// Sets a flag
  ///
  /// Wrapper around set flag to ensure cycles are correct
  fn set_flag(&mut self, flag: StatusBit) {
    self.status_register.set_flag(flag);
    // All instruction require at minimum two machine cycles
    self.sync();
  }

  /// Clears a flag
  ///
  /// Wrapper around clear flag to ensure cycles are correct
  fn clear_flag(&mut self, flag: StatusBit) {
    self.status_register.clear_flag(flag);
    // All instruction require at minimum two machine cycles
    self.sync();
  }

  fn convert_num_to_decimal(value: u8) -> std::result::Result<u8, std::num::ParseIntError> {
    let mut hi_nib = ((value & 0xF0) >> 4).to_string();
    let lo_nib = (value & 0x0F).to_string();
    hi_nib.push_str(&lo_nib);
    u8::from_str_radix(&hi_nib, 10)
  }

  fn decimal_addition(&mut self, acc_val: u8, val: u8, modifier: u8) -> u8 {
    trace!("Decimal addition. Hope this works!");
    let message = "D ADC";
    // Setup some closures to make life less painful
    let mod_result_acc = |v| (v & 0xFF) as u8;
    let mod_result_car = |v| (v & 0xFF00) > 1;
    let match_lo = |v| (v & 0x0F) as u16;
    let match_hi = |v| (v & 0xF0) as u16;
    // Get the low nibble of both values and add them with modifier
    // u16 so we don't deal with overflow yet
    let temp = match_lo(val) + match_lo(acc_val) + modifier as u16;
    let temp = match temp >= 0x0A {
      // If we're above the last valid decimal value, do some more modification
      true => ((temp + 0x06) & 0x0F) + 0x10,
      false => temp,
    };
    // Get the hi nibble of both values and add them to our lo nibble calc
    let result = (match_hi(val) + match_hi(acc_val)) as u16 + temp;
    // Handle N and V flags here BEFORE we proceed
    self.status_register.handle_n_flag(result as u8, message);
    self
      .status_register
      .handle_v_flag(mod_result_acc(result), message, mod_result_car(result));
    // If our hi nibble is above the last valid decimal value, additional modification
    let result = match result >= 0xA0 {
      true => result + 0x60,
      false => result,
    };
    let (result, carry) = (mod_result_acc(result), mod_result_car(result));
    self.status_register.handle_z_flag(result, message);
    self.status_register.handle_c_flag(message, carry);
    result
  }

  // If the carry is clear, modifier is 1
  fn decimal_subtraction(&mut self, acc_val: u8, val: u8, modifier: u8) -> u8 {
    trace!("Decimal subtraction. Hope this works!");
    let message = "D SBC";
    // Setup some closures to make life less painful
    let mod_result_acc = |v| (v & 0xFF) as u8;
    let mod_result_car = |v| !(v > acc_val && v > val);
    let match_lo = |v| v & 0x0F;
    let match_hi = |v| (v & 0xF0);
    // Get the low nibble of both values and subtract them with modifier
    let (temp, over1) = match_lo(acc_val).overflowing_sub(match_lo(val));
    let (temp, over2) = temp.overflowing_sub(modifier);
    // result is negative
    let temp = match over1 || over2 {
      true => ((temp - 0x06) & 0x0F).wrapping_sub(0x10),
      false => temp,
    };
    // Get the hi nibble of both values and subtract them to our lo nibble calc
    let (result, over1) = match_hi(acc_val).overflowing_sub(match_hi(val));
    let result = result.wrapping_add(temp);
    // Handle N and V flags here BEFORE we proceed
    self.status_register.handle_n_flag(result as u8, message);
    self
      .status_register
      .handle_v_flag(mod_result_acc(result), message, mod_result_car(result));
    // If our hi nibble is above the last valid decimal value, additional modification
    let result = match over1 {
      true => result - 0x60,
      false => result,
    };
    let (result, carry) = (mod_result_acc(result), mod_result_car(result));
    self.status_register.handle_z_flag(result, message);
    self.status_register.handle_c_flag(message, carry);
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
  /// AND byte with accumulator. If result is negative then carry is set.
  ///
  /// Affects flags N Z C. Carry is set if result is negative
  pub fn aac(&mut self, value: u8) {
    let message = "AAC";
    warn!("{} called. Something might be borked.", message);
    let result = value & self.accumulator.get();
    self.accumulator.set(result);
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_z_flag(result, message);
    self.status_register.handle_c_flag(
      message,
      self.status_register.is_flag_set(StatusBit::Negative),
    );
  }

  /// Illegal opcode.
  /// And x register with accumulator and store result in memory.
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
    let result = match self.status_register.is_flag_set(StatusBit::Decimal) {
      true => self.decimal_addition(self.accumulator.get(), value, modifier),
      false => {
        let first = self.accumulator.get().overflowing_add(value);
        let second = first.0.overflowing_add(modifier);
        let result = second.0;
        let carry = first.1 || second.1;
        self.status_register.handle_n_flag(result, message);
        self.status_register.handle_v_flag(result, message, carry);
        self.status_register.handle_z_flag(result, message);
        self.status_register.handle_c_flag(message, carry);
        result
      }
    };
    self.accumulator.set(result);
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
    debug!("BIT called checking {:X}", value_to_test);
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

  /// CLear Carry flag
  ///
  /// Clears the carry flag
  pub fn clc(&mut self) {
    debug!("CLC called");
    self.clear_flag(StatusBit::Carry);
  }

  /// CLear Decimal flag
  ///
  /// Clears the decimal flag
  pub fn cld(&mut self) {
    debug!("CLD called");
    self.clear_flag(StatusBit::Decimal);
  }

  /// CLear Interrupt flag
  ///
  /// Clears the interrupt flag
  pub fn cli(&mut self) {
    debug!("CLI called");
    self.clear_flag(StatusBit::Interrupt);
  }

  /// CLear oVerload flag
  ///
  /// Clears the overload flag
  pub fn clv(&mut self) {
    debug!("CLV called");
    self.clear_flag(StatusBit::Overflow);
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

  /// Zero page variant of DCP
  pub fn dcp_zp(&mut self) {
    trace!("DCP zero page called");
    let (index, value) = self.zero_page("DCP");
    self.dcp(index as u16, value);
  }

  /// Zero page x variant of DCP
  pub fn dcp_zp_reg(&mut self) {
    trace!("DCP zero page x called");
    let (index, value) = self.zp_reg("DEC", self.x_register.get());
    self.dcp(index as u16, value);
  }

  /// Absolute variant of DCP
  pub fn dcp_absolute(&mut self) {
    trace!("DCP absolute called");
    let (index, value) = self.absolute("DEC");
    self.dcp(index, value);
  }

  /// Absolute x variant of DCP
  pub fn dcp_abs_x(&mut self) {
    trace!("DCP absolute x called");
    let (index, value) = self.absolute_reg("DCP", self.x_register.get());
    self.dcp(index, value);
  }

  /// Absolute y variant of DCP
  pub fn dcp_abs_y(&mut self) {
    trace!("DCP absolute y called");
    let (index, value) = self.absolute_reg("DCP", self.y_register.get());
    self.dcp(index, value);
  }

  /// Indexed x variant of DCP
  pub fn dcp_indexed_x(&mut self) {
    trace!("DCP indexed x called");
    let (index, value) = self.indexed_x("DCP");
    self.dcp(index, value);
  }

  /// Indexed y variant of DCP
  pub fn dcp_indexed_y(&mut self) {
    trace!("DCP indexed y called");
    let (index, value) = self.indexed_y("DCP");
    self.dcp(index, value);
  }

  /// DECrement memory
  ///
  /// Gets memory from a source, decrements it by one, then saves it.
  /// Read Modify Update operation need an extra cycle to modify.
  pub fn dec(&mut self, index: u16, value: u8) {
    let value = value.wrapping_sub(1);
    // extra cycle for modification
    self.sync();
    debug!("DEC called index: {:X}, value: {:X}", index, value);
    self.set_u16(index, value);
    self.status_register.handle_n_flag(value, "DEC");
    self.status_register.handle_z_flag(value, "DEC");
  }

  /// Zero page variant of DEC
  pub fn dec_zp(&mut self) {
    trace!("DEC zero page called");
    let (index, value) = self.zero_page("DEC");
    self.dec(index as u16, value);
  }

  /// Zero page x variant of DEC
  pub fn dec_zp_reg(&mut self) {
    trace!("DEC zero page x called");
    let (index, value) = self.zp_reg("DEC", self.x_register.get());
    self.dec(index as u16, value);
  }

  /// Absolute variant of DEC
  pub fn dec_abs(&mut self) {
    trace!("DEC absolute called");
    let (index, value) = self.absolute("DEC");
    self.dec(index as u16, value);
  }

  /// Absolute x variant of DEC
  pub fn dec_abs_x(&mut self) {
    trace!("DEC absolute x called");
    let (index, value) = self.absolute_reg("DEC", self.x_register.get());
    self.dec(index, value);
    // extra cycle. do not know why
    self.sync();
  }

  /// Illegal opcode
  /// Nop. Underscored value to allow for callback variants
  pub fn dop(&mut self, _: u8) {
    warn!("DOP called. Something might be borked");
    self.nop();
  }

  /// Exclusive OR - more commonly known as XOR.
  ///
  /// Gets a value, XORs it, then stores that value in the accumulator
  pub fn eor(&mut self, value: u8) {
    let message = "EOR";
    debug!("{} called with value: 0x{:X}", message, value);
    let result = self.accumulator.get() ^ value;
    self.accumulator.set(result);
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_z_flag(result, message);
  }

  /// INCrement memory
  ///
  /// Gets a value from somewhere, decrements it, then stores it in memory.
  /// Read Modify Update operation need an extra cycle to modify.
  pub fn inc(&mut self, index: u16, value: u8) {
    let value = value.wrapping_add(1);
    // extra cycle for modification
    self.sync();
    debug!("INC called index: {:X}, value: {:X}", index, value);
    self.set_u16(index, value);
    self.status_register.handle_n_flag(value, "INC");
    self.status_register.handle_z_flag(value, "INC");
  }

  /// INC zero page variant
  pub fn inc_zp(&mut self) {
    trace!("INC zero page called");
    let (index, value) = self.zero_page("INC");
    self.inc(index as u16, value);
  }

  /// INC zero page x variant
  pub fn inc_zp_reg(&mut self) {
    trace!("INC zero page x called");
    let (index, value) = self.zp_reg("INC", self.x_register.get());
    self.inc(index as u16, value);
  }

  /// INC absolute variant
  pub fn inc_abs(&mut self) {
    trace!("INC absolute called");
    let (index, value) = self.absolute("INC");
    self.inc(index as u16, value);
  }

  /// INC absolute x variant
  pub fn inc_abs_x(&mut self) {
    trace!("INC absolute x called");
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

  /// JuMP
  ///
  /// Continues execution at a new value. Absolute variant.
  pub fn jmp_absolute(&mut self) {
    let ops = self.get_two_operands();
    let index = u16::from_le_bytes(ops);
    debug!("JMP absolute to index: {:X}", index);
    self.program_counter.jump(index);
  }

  /// JuMP
  ///
  /// Continues execution at a new value. Indirect variant
  pub fn jmp_indirect(&mut self) {
    let ops = self.get_two_operands();
    let (low_test, overflow) = ops[0].overflowing_add(1);
    if overflow {
      warn!("Indirect jump overflowing page. Results will be weird!");
    }
    let hi = self.get_u16(u16::from_le_bytes(ops));
    let lo = self.get_u16(u16::from_le_bytes([low_test, ops[1]]));
    let index = u16::from_le_bytes([hi, lo]);
    debug!("JMP indirect to index: {:X}", index);
    self.program_counter.jump(index);
  }

  /// Jump to SuRoutine
  ///
  /// Similar to a jump but to an explicit subroutine. Pushes the program
  /// counter to the stack to allow for returns.
  pub fn jsr(&mut self) {
    let ops = self.get_two_operands();
    self.program_counter.decrease(1);
    let pc_ops = self.program_counter.get().to_le_bytes();
    self.memory.push_to_stack(pc_ops[0]);
    self.memory.push_to_stack(pc_ops[1]);
    let index = u16::from_le_bytes(ops);
    debug!("JSR to index: {:X}, PC stored on stack", index,);
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

  /// LoaD Accumulator
  ///
  /// Loads a value into the accumulator
  ///
  /// Affects flags N Z
  pub fn lda(&mut self, value: u8) {
    let message = "LDA";
    debug!("{} called with value: 0x{:X}", message, value);
    self.accumulator.set(value);
    self.status_register.handle_n_flag(value, message);
    self.status_register.handle_z_flag(value, message);
  }

  /// LoaD X register
  ///
  /// Loads a value into the X register.
  ///
  /// Affects flags N Z
  pub fn ldx(&mut self, value: u8) {
    let message = "LDX";
    debug!("{} called with value: 0x{:X}", message, value);
    self.x_register.set(value);
    self.status_register.handle_n_flag(value, message);
    self.status_register.handle_z_flag(value, message);
  }

  /// LoaD Y register
  ///
  /// Loads a value into the Y register.
  ///
  /// Affects flags N Z
  pub fn ldy(&mut self, value: u8) {
    let message = "LDY";
    debug!("{} called with value: 0x{:X}", message, value);
    self.y_register.set(value);
    self.status_register.handle_n_flag(value, message);
    self.status_register.handle_z_flag(value, message);
  }

  /// Logical Shift Right
  ///
  /// Shifts all bits right one position
  ///
  /// Affects flags N Z C
  fn lsr(&mut self, value: u8) -> u8 {
    debug!("LSR called on {:X}", value);
    let (result, carry) = value.overflowing_shr(1);
    // extra cycle for modification
    self.sync();
    self.status_register.handle_n_flag(result, "LSR");
    self.status_register.handle_z_flag(result, "LSR");
    self.status_register.handle_c_flag("LSR", carry);
    result
  }

  /// LSR accumulator variant
  pub fn lsr_accumulator(&mut self) {
    trace!("LSR accumulator called");
    let result = self.lsr(self.accumulator.get());
    self.accumulator.set(result);
  }

  /// LSR zero page variant
  pub fn lsr_zero_page(&mut self) {
    trace!("LSR zero page called");
    let (index, value) = self.zero_page("LSR");
    let result = self.lsr(value);
    self.set_zero_page(index, result);
  }

  /// LSR zero page x variant
  pub fn lsr_zero_page_x(&mut self) {
    trace!("LSR zero page x called");
    let (index, value) = self.zp_reg("LSR", self.x_register.get());
    let result = self.lsr(value);
    self.set_zero_page(index, result);
  }

  /// LSR absolute variant
  pub fn lsr_absolute(&mut self) {
    trace!("LSR absolute called");
    let (index, value) = self.absolute("LSR");
    let result = self.lsr(value);
    self.set_u16(index, result);
  }

  /// LSR absolute x variant
  pub fn lsr_absolute_x(&mut self) {
    trace!("LSR absolute x called");
    let (index, value) = self.absolute_reg("LSR", self.x_register.get());
    let result = self.lsr(value);
    self.set_u16(index, result);
  }

  /// No OPeration
  ///
  /// Uses two cycles and gives you a smug sense of satisfaction
  pub fn nop(&mut self) {
    debug!("NOP called");
    // Extra cycle as all instruction require two bytes.
    self.sync();
  }

  /// OR with Accumulator
  ///
  /// Takes a value, ORs it with the accumulator, then sets that result to
  /// the accumulator
  pub fn ora(&mut self, value: u8) {
    let message = "ORA";
    debug!("{} called with value: 0x{:X}", message, value);
    let result = self.accumulator.get() | value;
    self.accumulator.set(result);
    self.status_register.handle_n_flag(result, message);
    self.status_register.handle_z_flag(result, message);
  }

  /// Transfer Accumulator to X register
  ///
  /// Takes the value in the accumulator and loads the x register with it.
  pub fn tax(&mut self) {
    self.x_register.set(self.x_register.get());
    self.register_operation(self.x_register.get(), "TAX");
  }

  /// Transfer X register to Accumulator
  ///
  /// Takes the value in the x register and loads the accumulator with it.
  pub fn txa(&mut self) {
    self.accumulator.set(self.x_register.get());
    self.register_operation(self.x_register.get(), "TXA");
  }

  /// DEcrement X register
  ///
  /// Takes the value in the x register and decrements it
  pub fn dex(&mut self) {
    self.x_register.decrement();
    self.register_operation(self.x_register.get(), "DEX");
  }

  /// INcrement X register
  ///
  /// Takes the value in the x register and increments it
  pub fn inx(&mut self) {
    self.x_register.increment();
    self.register_operation(self.x_register.get(), "INX");
  }

  /// Transfer Accumulator to Y register
  ///
  /// Takes the value in the accumulator and loads the Y register with it
  pub fn tay(&mut self) {
    self.y_register.set(self.accumulator.get());
    self.register_operation(self.y_register.get(), "TAY");
  }

  /// Transfer Y register to Accumulator
  ///
  /// Takes the value in the Y register and loads the accumulator with it
  pub fn tya(&mut self) {
    self.accumulator.set(self.y_register.get());
    self.register_operation(self.y_register.get(), "TYA");
  }

  /// DEcrement Y register
  ///
  /// Takes the value in the Y register and decrements it.
  pub fn dey(&mut self) {
    self.y_register.decrement();
    self.register_operation(self.y_register.get(), "DEY");
  }

  /// INcrement Y register
  ///
  /// Takes the value in the Y register and increments it.
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

  /// ROtate Left
  ///
  /// Takes a value and rotates bits to the left.
  fn rol(&mut self, value: u8) -> u8 {
    debug!("ROL called with value: {:X}", value);
    let result = self.rotate_left(value);
    // extra cycle for modification
    self.sync();
    self.status_register.handle_n_flag(result, "ROL");
    self.status_register.handle_z_flag(result, "ROL");
    result
  }

  /// Rotate left accumulator variant.
  pub fn rol_accumulator(&mut self) {
    trace!("ROL accumulator called");
    let result = self.rol(self.accumulator.get());
    self.accumulator.set(result);
  }

  /// Rotate left zero page variant
  pub fn rol_zero_page(&mut self) {
    trace!("ROL zero page called");
    let (index, value) = self.zero_page("ROL");
    let result = self.rol(value);
    self.set_zero_page(index, result);
  }

  /// Rotate left zero page x variant
  pub fn rol_zero_page_x(&mut self) {
    trace!("ROL zero page x variant");
    let (index, value) = self.zp_reg("ROL", self.x_register.get());
    let result = self.rol(value);
    self.set_zero_page(index, result);
  }

  /// Rotate left absolute variant
  pub fn rol_absolute(&mut self) {
    trace!("ROL absolute variant");
    let (index, value) = self.absolute("ROL");
    let result = self.rol(value);
    self.set_u16(index, result);
  }

  /// Rotate left absolute x variant
  pub fn rol_absolute_x(&mut self) {
    trace!("ROL absolute x variant");
    let (index, value) = self.absolute_reg("ROL", self.x_register.get());
    let result = self.rol(value);
    self.set_u16(index, result);
  }

  /// ROtate Right
  ///
  /// Takes a value and rotates bits to the right.
  fn ror(&mut self, value: u8) -> u8 {
    debug!("ROR called with value: {:X}", value);
    let result = self.rotate_right(value);
    // extra cycle for modification
    self.sync();
    self.status_register.handle_n_flag(result, "ROR");
    self.status_register.handle_z_flag(result, "ROR");
    result
  }

  /// Rotate right accumulator variant
  pub fn ror_accumulator(&mut self) {
    trace!("ROR accumulator called");
    let result = self.ror(self.accumulator.get());
    self.accumulator.set(result);
  }

  /// Rotate right zero page variant
  pub fn ror_zero_page(&mut self) {
    trace!("ROR zero page called");
    let (index, value) = self.zero_page("ROR");
    let result = self.ror(value);
    self.set_zero_page(index, result);
  }

  /// Rotate right zero page x variant
  pub fn ror_zero_page_x(&mut self) {
    trace!("ROR zero page x called");
    let (index, value) = self.zp_reg("ROR", self.x_register.get());
    let result = self.ror(value);
    self.set_zero_page(index, result);
  }

  /// Rotate right absolute variant
  pub fn ror_absolute(&mut self) {
    trace!("ROR absolute called");
    let (index, value) = self.absolute("ROR");
    let result = self.ror(value);
    self.set_u16(index, result);
  }

  /// Rotate right absolute x variant
  pub fn ror_absolute_x(&mut self) {
    trace!("ROR absolute x called");
    let (index, value) = self.absolute_reg("ROR", self.x_register.get());
    let result = self.ror(value);
    self.set_u16(index, result);
  }

  /// Illegal opcode.
  /// Rotate one bit right in memory, then add memory to accumulator (with
  /// carry).
  ///
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

  /// ReTurn from Interrupt
  ///
  /// Returns from an interrupt when called.
  pub fn rti(&mut self) {
    debug!("RTI called");
    self.return_from_interrupt();
  }

  /// ReTurn from Subroutine
  ///
  /// Returns from a subroutine. Retrieves the program counter value from the
  /// stack and sets the program counter to it.
  pub fn rts(&mut self) {
    debug!("RTS called");
    let lo = self.pop_from_stack();
    let hi = self.pop_from_stack();
    let index = u16::from_le_bytes([lo, hi]) + 1;
    // extra cycle to increment the index
    self.sync();
    self.program_counter.jump(index);
    // one byte extra cycle
    self.sync();
  }

  /// SuBtract with Carry
  ///
  /// Takes a value and subtracts it from the accumulator. Affected by carry.
  ///
  /// Affects flags N V Z C
  pub fn sbc(&mut self, value: u8) {
    let message = "SBC";
    debug!("{} called with value: 0x{:X}", message, value);
    let modifier = match self.status_register.is_flag_set(StatusBit::Carry) {
      true => 0,
      false => 1,
    };
    let result = match self.status_register.is_flag_set(StatusBit::Decimal) {
      true => self.decimal_subtraction(self.accumulator.get(), value, modifier),
      false => {
        let (val, acc) = (value, self.accumulator.get());
        let first = acc.overflowing_sub(val);
        let second = first.0.overflowing_sub(modifier);
        let result = second.0;
        let carry = first.1 || second.1;
        self.status_register.handle_n_flag(result, message);
        self.status_register.handle_v_flag(result, message, carry);
        self.status_register.handle_z_flag(result, message);
        self.status_register.handle_c_flag(message, carry);
        result
      }
    };
    self.accumulator.set(result);
  }

  /// SEt Carry flag
  ///
  /// Sets the carry flag
  pub fn sec(&mut self) {
    debug!("SEC called");
    self.set_flag(StatusBit::Carry);
  }

  /// SEt Decimal flag
  ///
  /// Sets the decimal flag
  pub fn sed(&mut self) {
    debug!("SEC called");
    self.set_flag(StatusBit::Decimal);
  }

  /// SEt Interrupt flag
  ///
  /// Sets the interrupt flag
  pub fn sei(&mut self) {
    self.set_flag(StatusBit::Interrupt);
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

  /// STore Accumulator
  ///
  /// Stores a value in the accumulator, zero page variant
  pub fn sta_zero_page(&mut self) {
    debug!("STA zero page called");
    let index = self.zero_page_index("STA");
    self.set_zero_page(index, self.accumulator.get());
  }

  /// STore Accumulator
  ///
  /// Stores a value in the accumulator, zero page x variant
  pub fn sta_zero_page_x(&mut self) {
    debug!("STA zero page called");
    let index = self.zp_reg_index("STA", self.x_register.get());
    self.set_zero_page(index, self.accumulator.get());
  }

  /// STore Accumulator
  ///
  /// Stores a value in the accumulator, absolute variant
  pub fn sta_absolute(&mut self) {
    debug!("STA absolute called");
    let index = self.absolute_index("STA");
    self.set_u16(index, self.accumulator.get());
  }

  /// STore Accumulator
  ///
  /// Stores a value in the accumulator, absolute x variant
  pub fn sta_absolute_x(&mut self) {
    debug!("STA absolute x called");
    let (index, _) = self.absolute_reg("STA", self.x_register.get());
    self.set_u16(index, self.accumulator.get());
  }

  /// STore Accumulator
  ///
  /// Stores a value in the accumulator, absolute y variant
  pub fn sta_absolute_y(&mut self) {
    debug!("STA absolute y called");
    let (index, _) = self.absolute_reg("STA", self.y_register.get());
    self.set_u16(index, self.accumulator.get());
  }

  /// STore Accumulator
  ///
  /// Stores a value in the accumulator, indexed x variant
  pub fn sta_indexed_x(&mut self) {
    debug!("STA indexed x called");
    let (index, _) = self.indexed_x("STA");
    self.set_u16(index, self.accumulator.get());
  }

  /// STore Accumulator
  ///
  /// Stores a value in the accumulator, indexed y variant
  pub fn sta_indexed_y(&mut self) {
    debug!("STA indexed y called");
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

  /// Transfer X register to Stack pointer
  ///
  /// Takes the value in the x register and loads the stack pointer with it
  pub fn txs(&mut self) {
    debug!("TXS called");
    self.memory.set_stack_pointer(self.x_register.get());
    // extra instruction byte always happens
    self.sync();
  }

  /// Transfer Stack pointer to X register
  ///
  /// Takes the value in the stack pointer and loads the x register with it
  pub fn tsx(&mut self) {
    debug!("TSX called");
    self.x_register.set(self.memory.get_stack_pointer().get());
    // extra instruction byte always happens
    self.sync();
  }

  /// PusH Accumulator
  ///
  /// Pushes the accumulator value to the stack
  pub fn pha(&mut self) {
    debug!("PHA called");
    self.push_to_stack(self.accumulator.get());
    // extra instruction byte always happens
    self.sync();
  }

  /// PulL Accumulator
  ///
  /// Pops the stack value and sets the accumulator to it
  /// In 6502 parlance Pull means Pop from the stack.
  pub fn pla(&mut self) {
    debug!("PLA called");
    let stack_value = self.pop_from_stack();
    self.accumulator.set(stack_value);
    // extra instruction byte always happens
    self.sync();
  }

  /// PusH Processor status
  ///
  /// Pushes the status register onto the stack
  pub fn php(&mut self) {
    debug!("PHP called");
    self.push_to_stack(self.status_register.get_register());
    // extra instruction byte always happens
    self.sync();
  }

  /// PulL Processor status
  ///
  /// Pops the stack value and sets the status register to it
  /// In 6502 parlance Pull means Pop from the stack.
  pub fn plp(&mut self) {
    debug!("PLP called");
    let stack = self.pop_from_stack();
    self.status_register.set(stack);
    // extra instruction byte always happens
    self.sync();
  }

  /// STore X register
  ///
  /// Takes the value in the x register and stores it in memory.
  /// Zero page variant
  pub fn stx_zero_page(&mut self) {
    let index = self.zero_page_index("STX");
    self.set_zero_page(index, self.x_register.get());
  }

  /// STore X register
  ///
  /// Takes the value in the x register and stores it in memory.
  /// Zero page y variant
  pub fn stx_zero_page_y(&mut self) {
    let index = self.zp_reg_index("STX", self.y_register.get());
    self.set_zero_page(index, self.x_register.get());
  }

  /// STore X register
  ///
  /// Takes the value in the x register and stores it in memory.
  /// Absolute variant
  pub fn stx_absolute(&mut self) {
    let index = self.absolute_index("STX");
    self.set_u16(index, self.x_register.get());
  }

  /// STore Y register
  ///
  /// Takes the value in the y register and stores it in memory.
  /// Zero page variant
  pub fn sty_zero_page(&mut self) {
    let index = self.zero_page_index("STY");
    self.set_zero_page(index, self.y_register.get());
  }

  /// STore Y register
  ///
  /// Takes the value in the x register and stores it in memory.
  /// Zero page x variant
  pub fn sty_zero_page_x(&mut self) {
    let index = self.zp_reg_index("STY", self.x_register.get());
    self.set_zero_page(index, self.y_register.get());
  }

  /// STore Y register
  ///
  /// Takes the value in the x register and stores it in memory.
  /// Absolute variant
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

/// Prints pretty output about the status of the CPU.
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
  use rand::random;
  use rand::Rng;
  use std::sync::mpsc;
  use test_case::test_case;

  fn new_cpu() -> CPU {
    let (_, rx) = mpsc::channel();
    CPU::new(rx)
  }

  fn setup_sync(count: usize) -> CPU {
    let (tx, rx) = mpsc::channel();
    let cpu = CPU::new(rx);
    std::thread::spawn(move || {
      for _ in 0..count {
        tx.send(true).unwrap();
      }
    });
    cpu
  }

  fn wrapping_u8() -> u8 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0x80, 0xFF)
  }

  fn non_wrapping_u8() -> u8 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0x00, 0x7F)
  }

  fn wrapping_u16() -> u16 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0x8000, 0xFFFF)
  }

  fn non_wrapping_u16() -> u16 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0x0000, 0x7FFF)
  }

  #[test]
  fn new() {
    let (_, rx) = mpsc::channel();
    let cpu = CPU::new(rx);
    assert_eq!(cpu.program_counter.get(), STARTING_MEMORY_BLOCK as usize);
    assert_eq!(cpu.accumulator.get(), 0);
    assert_eq!(cpu.x_register.get(), 0);
    assert_eq!(cpu.y_register.get(), 0);
    assert_eq!(cpu.status_register.get_register(), 0);
    assert_eq!(cpu.memory.get_u16(random()), 0);
    assert_eq!(cpu.reset_pin, false);
    assert_eq!(cpu.irq_pin, false);
    assert_eq!(cpu.nmi_pin, false);
  }

  #[test]
  fn reset() {
    let mut cpu = new_cpu();
    cpu.program_counter.jump(random());
    cpu.accumulator.set(random());
    cpu.x_register.set(random());
    cpu.y_register.set(random());
    cpu.status_register.set(random());
    cpu.memory.set(random(), random());
    cpu.reset_pin = true;
    cpu.irq_pin = true;
    cpu.nmi_pin = true;
    cpu.reset();
    assert_eq!(cpu.program_counter.get(), STARTING_MEMORY_BLOCK as usize);
    assert_eq!(cpu.accumulator.get(), 0);
    assert_eq!(cpu.x_register.get(), 0);
    assert_eq!(cpu.y_register.get(), 0);
    assert_eq!(cpu.status_register.get_register(), 0);
    assert_eq!(cpu.memory.get_u16(random()), 0);
    assert_eq!(cpu.reset_pin, false);
    assert_eq!(cpu.irq_pin, false);
    assert_eq!(cpu.nmi_pin, false);
  }

  #[test]
  fn load_program_into_memory() {
    let mut cpu = new_cpu();
    let mut vector = vec![];
    for _ in 0..0xFF {
      vector.push(wrapping_u8());
    }
    cpu.load_program_into_memory(&vector, 0);
    assert_eq!(cpu.memory.get_zero_page(0x92) > 0, true);
  }

  #[test]
  #[should_panic]
  fn load_program_into_memory_panic() {
    let mut cpu = new_cpu();
    let mut vector = vec![];
    for _ in 0..0xFF {
      vector.push(random());
    }
    cpu.load_program_into_memory(&vector, 0xFFFF);
  }

  #[test]
  fn sync_proceeds_when_clock_signal_received() {
    let mut cpu = setup_sync(1);
    cpu.sync();
    // if we got here, things are working
    assert_eq!(true, true);
  }

  #[test_case(random())]
  fn push_to_stack(value: u8) {
    let mut cpu = setup_sync(1);
    cpu.push_to_stack(value);
    assert_eq!(cpu.memory.get_u16(0x1FF), value);
  }

  #[test_case(random())]
  fn pop_from_stack(value: u8) {
    let mut cpu = setup_sync(2);
    cpu.memory.set(0x100, value);
    let result = cpu.pop_from_stack();
    assert_eq!(result, value);
  }

  #[test_case(random(), random())]
  fn set_u16(index: u16, value: u8) {
    let mut cpu = setup_sync(1);
    cpu.set_u16(index, value);
    assert_eq!(cpu.memory.get_u16(index), value);
  }

  #[test_case(random(), random())]
  fn set_zero_page(index: u8, value: u8) {
    let mut cpu = setup_sync(1);
    cpu.set_zero_page(index, value);
    assert_eq!(cpu.memory.get_zero_page(index), value);
  }

  #[test_case(random())]
  fn get_single_operand(value: u8) {
    let mut cpu = setup_sync(1);
    let pc = cpu.program_counter.get() + 1;
    cpu.memory.set(pc as u16, value);
    let op = cpu.get_single_operand();
    assert_eq!(op, value);
  }

  #[test_case(random(), random())]
  fn get_two_operands(v1: u8, v2: u8) {
    let mut cpu = setup_sync(2);
    let pc = cpu.program_counter.get() + 1;
    cpu.memory.set(pc as u16, v1);
    cpu.memory.set((pc + 1) as u16, v2);
    let ops = cpu.get_two_operands();
    assert_eq!(ops[0], v1);
    assert_eq!(ops[1], v2);
  }

  #[test_case(non_wrapping_u8(), non_wrapping_u8(), 0; "Non wrap")]
  #[test_case(wrapping_u8(), wrapping_u8(), 1; "Wrap")]
  fn test_for_overflow(v1: u8, v2: u8, sync_count: usize) {
    let mut cpu = setup_sync(sync_count);
    cpu.test_for_overflow(v1, v2);
    // if we're here, things worked
    assert_eq!(true, true);
  }

  // NOTES FOR SYNC COUNTS IN THIS SECTION
  // Syncs are not counting the initial opcode read, so all sync counts
  // are one less than in the actual execution.

  #[test_case(random())]
  fn immediate(value: u8) {
    let mut cpu = setup_sync(1);
    let pc = cpu.program_counter.get();
    cpu.memory.set((pc + 1) as u16, value);
    let result = cpu.immediate("Test");
    assert_eq!(value, result);
  }

  #[test_case(random(), random())]
  fn zero_page(value: u8, index: u8) {
    let mut cpu = setup_sync(2);
    let pc = cpu.program_counter.get();
    cpu.memory.set((pc + 1) as u16, index);
    cpu.memory.set_zero_page(index, value);
    let (i_result, v_result) = cpu.zero_page("Test");
    assert_eq!(value, v_result);
    assert_eq!(index, i_result);
  }

  #[test_case(random(), 0x10, 0x20; "No wrap")]
  #[test_case(random(), wrapping_u8(), wrapping_u8(); "Wrap")]
  fn zero_page_reg(value: u8, index: u8, reg: u8) {
    let mut cpu = setup_sync(3);
    let pc = cpu.program_counter.get();
    cpu.memory.set((pc + 1) as u16, index);
    let index = index.wrapping_add(reg);
    cpu.memory.set_zero_page(index, value);
    let (i_result, v_result) = cpu.zp_reg("Test", reg);
    assert_eq!(value, v_result);
    assert_eq!(index, i_result);
  }

  #[test_case(random(), random())]
  fn absolute(value: u8, index: u16) {
    let mut cpu = setup_sync(3);
    let pc = cpu.program_counter.get();
    let ops = index.to_le_bytes();
    cpu.memory.set((pc + 1) as u16, ops[0]);
    cpu.memory.set((pc + 2) as u16, ops[1]);
    cpu.memory.set(index, value);
    let (i_result, v_result) = cpu.absolute("Test");
    assert_eq!(value, v_result);
    assert_eq!(index, i_result);
  }

  #[test_case(random(), non_wrapping_u16(), non_wrapping_u8(), 3; "Non wrapping")]
  #[test_case(random(), wrapping_u16(), wrapping_u8(), 4; "Wrapping")]
  fn absolute_reg(value: u8, index: u16, reg: u8, sync_count: usize) {
    let mut cpu = setup_sync(sync_count);
    let pc = cpu.program_counter.get();
    let ops = index.to_le_bytes();
    cpu.memory.set((pc + 1) as u16, ops[0]);
    cpu.memory.set((pc + 2) as u16, ops[1]);
    let index = index.wrapping_add(reg as u16);
    cpu.memory.set(index, value);
    let (i_result, v_result) = cpu.absolute_reg("Test", reg);
    assert_eq!(value, v_result);
    assert_eq!(index, i_result);
  }

  #[test_case(random(), non_wrapping_u16(), non_wrapping_u8(), non_wrapping_u8(), 5; "No wrap")]
  #[test_case(random(), wrapping_u16(), wrapping_u8(), wrapping_u8(), 5; "Wrap")]
  fn indexed_x(value: u8, index: u16, reg: u8, op: u8, sync_count: usize) {
    let mut cpu = setup_sync(sync_count);
    let pc = cpu.program_counter.get();
    cpu.x_register.set(reg);
    cpu.memory.set((pc + 1) as u16, op);
    let mod_op = op.wrapping_add(reg);
    let ops = index.to_le_bytes();
    cpu.memory.set_zero_page(mod_op, ops[0]);
    cpu.memory.set_zero_page(mod_op.wrapping_add(1), ops[1]);
    cpu.memory.set(index, value);
    let (i_result, v_result) = cpu.indexed_x("test");
    assert_eq!(value, v_result);
    assert_eq!(index, i_result);
  }

  #[test_case(random(), non_wrapping_u16(), non_wrapping_u8(), non_wrapping_u8(), 4; "No wrap")]
  #[test_case(random(), wrapping_u16(), wrapping_u8(), wrapping_u8(), 5; "Wrap")]
  fn indexed_y(value: u8, index: u16, reg: u8, op: u8, sync_count: usize) {
    let mut cpu = setup_sync(sync_count);
    let pc = cpu.program_counter.get();
    cpu.y_register.set(reg);
    cpu.memory.set((pc + 1) as u16, op);
    let ops = index.to_le_bytes();
    cpu.memory.set_zero_page(op, ops[0]);
    cpu.memory.set_zero_page(op.wrapping_add(1), ops[1]);
    let index = index.wrapping_add(reg as u16);
    cpu.memory.set(index, value);
    let (i_result, v_result) = cpu.indexed_y("test");
    assert_eq!(value, v_result);
    assert_eq!(index, i_result);
  }

  #[test_case(true, non_wrapping_u8(), 1; "Addition")]
  #[test_case(true, wrapping_u8(), 2; "Subtraction")]
  fn branch(condition: bool, op: u8, sync_count: usize) {
    let mut cpu = setup_sync(sync_count);
    let pc = cpu.program_counter.get();
    cpu.branch(condition, op);
    let result = match op > 0x7F {
      true => pc - !(op + 1) as usize,
      false => pc + op as usize,
    };
    assert_eq!(result, cpu.program_counter.get());
  }

  #[test]
  fn branch_not_taken() {
    let mut cpu = setup_sync(0);
    cpu.branch(false, 0x15);
    assert_eq!(cpu.program_counter.get(), STARTING_MEMORY_BLOCK as usize);
  }

  #[test_case(0x10, 0x20; "Not equal, negative, carried")]
  #[test_case(0x20, 0x10; "Not equal, positive, not carried")]
  #[test_case(0x20, 0x20; "Equal, positive, carried")]
  #[test_case(0x01, 0xFF; "Not equal, positive, carried")]
  fn generic_compare(reg_value: u8, test_value: u8) {
    let mut cpu = setup_sync(0);
    cpu.generic_compare(test_value, reg_value);
    assert_eq!(
      cpu.status_register.is_flag_set(StatusBit::Carry),
      reg_value >= test_value
    );
    assert_eq!(
      cpu.status_register.is_flag_set(StatusBit::Zero),
      test_value == reg_value
    );
    assert_eq!(
      cpu.status_register.is_flag_set(StatusBit::Negative),
      reg_value.wrapping_sub(test_value) >= 0x80
    );
  }

  #[test_case(0; "Zero")]
  #[test_case(0xAA; "Negative")]
  #[test_case(0x12; "Positive")]
  fn register_operation(value: u8) {
    let mut cpu = setup_sync(1);
    cpu.register_operation(value, "test");
    assert_eq!(
      cpu.status_register.is_flag_set(StatusBit::Negative),
      value >= 0x80
    );
    assert_eq!(
      cpu.status_register.is_flag_set(StatusBit::Zero),
      value == 0x00
    );
  }

  #[test_case(0x10, false, 0x8, false; "Normal no mods")]
  #[test_case(0x10, true, 0x88, false; "With carry set prior")]
  #[test_case(0x51, false, 0x28, true; "Expecting to carry")]
  #[test_case(0x51, true, 0xA8, true; "With carry set and expecting to carry")]
  fn rotate_right(value: u8, set_carry: bool, expected: u8, expected_carry: bool) {
    let mut cpu = setup_sync(0);
    if set_carry {
      cpu.status_register.set_flag(StatusBit::Carry);
    }
    let result = cpu.rotate_right(value);
    assert_eq!(
      cpu.status_register.is_flag_set(StatusBit::Carry),
      expected_carry
    );
    assert_eq!(result, expected);
  }

  #[test_case(0x10, 0x8, false; "Normal no mods")]
  #[test_case(0x51, 0x28, true; "Expecting to carry")]
  fn shift_right(value: u8, expected: u8, expected_carry: bool) {
    let mut cpu = setup_sync(0);
    let result = cpu.shift_right(value);
    assert_eq!(
      cpu.status_register.is_flag_set(StatusBit::Carry),
      expected_carry
    );
    assert_eq!(result, expected);
  }

  #[test_case(0x10, false, 0x20, false; "Normal no mods")]
  #[test_case(0x10, true, 0x21, false; "With carry set prior")]
  #[test_case(0x91, false, 0x22, true; "Expecting to carry")]
  #[test_case(0x91, true, 0x23, true; "With carry set and expecting to carry")]
  fn rotate_left(value: u8, set_carry: bool, expected: u8, expected_carry: bool) {
    let mut cpu = setup_sync(0);
    if set_carry {
      cpu.status_register.set_flag(StatusBit::Carry);
    }
    let result = cpu.rotate_left(value);
    assert_eq!(
      cpu.status_register.is_flag_set(StatusBit::Carry),
      expected_carry
    );
    assert_eq!(result, expected);
  }

  #[test_case(0x10, 0x20, false; "Normal no mods")]
  #[test_case(0x91, 0x22, true; "Expecting to carry")]
  fn shift_left(value: u8, expected: u8, expected_carry: bool) {
    let mut cpu = setup_sync(0);
    let result = cpu.shift_left(value);
    assert_eq!(
      cpu.status_register.is_flag_set(StatusBit::Carry),
      expected_carry
    );
    assert_eq!(result, expected);
  }

  #[test_case(StatusBit::Carry; "Set carry")]
  #[test_case(StatusBit::Zero; "Set zero")]
  #[test_case(StatusBit::Negative; "Set negative")]
  #[test_case(StatusBit::Decimal; "Set decimal")]
  #[test_case(StatusBit::Overflow; "Set overflow")]
  #[test_case(StatusBit::Interrupt; "Set interrupt")]
  #[test_case(StatusBit::Break; "Set break")]
  fn set_flag(flag: StatusBit) {
    let mut cpu = setup_sync(1);
    cpu.set_flag(flag);
    assert_eq!(cpu.status_register.is_flag_set(flag), true);
  }

  #[test_case(StatusBit::Carry; "Clear carry")]
  #[test_case(StatusBit::Zero; "Clear zero")]
  #[test_case(StatusBit::Negative; "Clear negative")]
  #[test_case(StatusBit::Decimal; "Clear decimal")]
  #[test_case(StatusBit::Overflow; "Clear overflow")]
  #[test_case(StatusBit::Interrupt; "Clear interrupt")]
  #[test_case(StatusBit::Break; "Clear break")]
  fn clear_flag(flag: StatusBit) {
    let mut cpu = setup_sync(1);
    cpu.status_register.set_flag(flag);
    cpu.clear_flag(flag);
    assert_eq!(cpu.status_register.is_flag_set(flag), false);
  }

  // #[test_case(0x19, 19)]
  // #[test_case(0x26, 26)]
  // #[test_case(0x71, 71)]
  // fn convert_num_to_decimal(hex_num: u8, dec_num: u8) {
  //   let result = CPU::convert_num_to_decimal(hex_num);
  //   assert_eq!(result, dec_num);
  // }

  // #[test_case(0xA2)]
  // #[test_case(0x2A)]
  // #[should_panic]
  // fn convert_num_to_decimal_panic(value: u8) {
  //   let result = CPU::convert_num_to_decimal(value);
  //   assert_eq!(0, result + 1);
  // }

  #[test_case(0xFFFA, 0xFFFB, random(), random(), random(), random(); "interrupt values")]
  fn interrupt(lo: u16, hi: u16, v1: u8, v2: u8, sr: u8, pc: u16) {
    let mut cpu = setup_sync(7);
    cpu.memory.set(lo, v1);
    cpu.memory.set(hi, v2);
    cpu.status_register.set(sr);
    cpu.program_counter.jump(pc);
    let pc_ops = pc.to_le_bytes();
    let address = cpu.interrupt(lo, hi);
    assert_eq!(address, u16::from_le_bytes([v1, v2]));
    assert_eq!(cpu.memory.get_u16(0x1FF), pc_ops[0]);
    assert_eq!(cpu.memory.get_u16(0x1FE), pc_ops[1]);
    assert_eq!(
      cpu.memory.get_u16(0x1FD),
      cpu.status_register.get_register()
    );
    assert_eq!(cpu.status_register.is_flag_set(StatusBit::Interrupt), true);
  }

  // 0xCF is all flags except break and unused set
  #[test_case(0xCF, random())]
  fn return_from_interrupt(sr: u8, pc: u16) {
    let mut cpu = setup_sync(6);
    cpu.program_counter.jump(pc);
    let pc_ops = pc.to_le_bytes();
    cpu.memory.set(0x1FF, pc_ops[0]);
    cpu.memory.set(0x1FE, pc_ops[1]);
    cpu.memory.set(0x1FD, sr);
    cpu.memory.set_stack_pointer(0xFC);
    cpu.return_from_interrupt();
    assert_eq!(cpu.program_counter.get(), pc as usize);
    assert_eq!(cpu.status_register.get_register(), 0xCB);
    assert_eq!(cpu.status_register.is_flag_set(StatusBit::Interrupt), false);
  }

  #[test_case(0x58, 0x46, 1, 0x05, true)]
  #[test_case(0x12, 0x34, 0, 0x46, false)]
  #[test_case(0x15, 0x26, 0, 0x41, false)]
  #[test_case(0x81, 0x92, 0, 0x73, true)]
  fn decimal_addition(v1: u8, v2: u8, carry: u8, expected: u8, carry_set: bool) {
    let mut cpu = setup_sync(0);
    let result = cpu.decimal_addition(v1, v2, carry);
    assert_eq!(result, expected);
    assert_eq!(cpu.status_register.is_flag_set(StatusBit::Carry), carry_set);
  }

  #[test_case(0x46, 0x12, 1, 0x34, true)]
  #[test_case(0x40, 0x13, 1, 0x27, true)]
  #[test_case(0x32, 0x02, 0, 0x29, true)]
  #[test_case(0x12, 0x21, 1, 0x91, false)]
  #[test_case(0x21, 0x34, 1, 0x87, false)]
  fn decimal_subtraction(v1: u8, v2: u8, carry: u8, expected: u8, carry_set: bool) {
    let mut cpu = setup_sync(0);
    // carries are flipped for subtraction
    let carry = match carry {
      1 => 0,
      0 => 1,
      _ => panic!(),
    };
    let result = cpu.decimal_subtraction(v1, v2, carry);
    assert_eq!(result, expected);
    assert_eq!(cpu.status_register.is_flag_set(StatusBit::Carry), carry_set);
  }

  #[test_case(random(), random())]
  fn aac(value: u8, acc: u8) {
    let mut cpu = setup_sync(0);
    cpu.accumulator.set(acc);
    cpu.aac(value);
    let result = value & acc;
    assert_eq!(cpu.accumulator.get(), result);
    assert_eq!(
      cpu.status_register.is_flag_set(StatusBit::Carry),
      result >= 0x80
    );
  }

  #[test_case(random(), random(), random())]
  fn aax(index: u16, acc: u8, x: u8) {
    let mut cpu = setup_sync(1);
    cpu.x_register.set(x);
    cpu.accumulator.set(acc);
    cpu.aax(index);
    assert_eq!(cpu.memory.get_u16(index), x & acc);
  }

  #[test_case(0x58, 0x46, true, false, 0x9F; "test hex addition")]
  #[test_case(0x58, 0x46, true, true, 0x05; "test dec addition")]
  fn adc(acc: u8, value: u8, c: bool, d: bool, expected: u8) {
    let mut cpu = setup_sync(0);
    cpu.accumulator.set(acc);
    if c {
      cpu.status_register.set_flag(StatusBit::Carry);
    }
    if d {
      cpu.status_register.set_flag(StatusBit::Decimal);
    }
    cpu.adc(value);
    assert_eq!(cpu.accumulator.get(), expected);
  }
}
