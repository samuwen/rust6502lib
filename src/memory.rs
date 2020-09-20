use crate::StackPointer;
use log::error;

/// 16 bits worth of screaming fast memory.
const MEMORY_MAX: usize = 0xFFFF;
/// The start block for the stack
const STACK_MIN: u16 = 0x100;
/// The end block for the stack
const STACK_MAX: u16 = 0x1FF;

/// Simulation of 16bit memory for 6502. Basically just an array with some
/// mapping functions. Contains the stack pointer to make things convenient.
pub struct Memory {
  mem: [u8; MEMORY_MAX],
  sp: StackPointer,
}

impl Memory {
  /// Creates and initializes a new Memory object.
  pub fn new() -> Memory {
    Memory {
      mem: [0; MEMORY_MAX],
      sp: StackPointer::new(),
    }
  }

  /// Resets the memory to the base state.
  pub fn reset(&mut self) {
    self.mem = [0; MEMORY_MAX];
    self.sp.reset();
  }

  /// Sets an index to a value. Logs an error if this overwrites the stack.
  pub fn set(&mut self, index: u16, value: u8) {
    if index <= STACK_MAX && index >= STACK_MIN {
      error!("Accessing memory from the stack improperly!");
    }
    self.mem[index as usize] = value;
  }

  /// Sets memory in the zero page. This takes less machine cycles than a normal write
  /// so we have a specific method to preserve cycle timing.
  pub fn set_zero_page(&mut self, index: u8, value: u8) {
    self.mem[index as usize] = value;
  }

  /// Gets memory from the zero page. This takes less machine cycles than a normal read
  /// so we have a sepcific method to preserve cycle timing.
  pub fn get_zero_page(&self, index: u8) -> u8 {
    self.mem[index as usize]
  }

  /// Gets the value at an index. Logs an error if this reads from the stack.
  pub fn get_u16(&self, index: u16) -> u8 {
    if index <= STACK_MAX && index >= STACK_MIN {
      error!("Accessing memory from the stack improperly!");
    }
    self.mem[index as usize]
  }

  /// Adds a value to the stack. Takes in a value to be entered and the stack pointer.
  /// We store our stack pointer as a u8, and our stack index starts at 0x100. So we
  /// logical OR the pointer val with 0x100 to get a value between 0x100 & 0x1FF.
  pub fn push_to_stack(&mut self, value: u8) {
    let index = (STACK_MIN | self.sp.push()) as usize;
    self.mem[index] = value;
  }

  /// Takes a value from the stack. Returns the value at the current stack pointer.
  /// We store our stack pointer as a u8, and our stack index starts at 0x100. So we
  /// logical OR the pointer val with 0x100 to get a value between 0x100 & 0x1FF.
  pub fn pop_from_stack(&mut self) -> u8 {
    let index = (STACK_MIN | self.sp.pop()) as usize;
    self.mem[index]
  }

  /// Gets our instance of the stack pointer
  pub fn get_stack_pointer(&self) -> &StackPointer {
    &self.sp
  }

  /// Sets our instance of the stack pointer
  pub fn set_stack_pointer(&mut self, val: u8) {
    self.sp.set(val);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rand::random;
  use test_case::test_case;

  #[test]
  fn new() {
    let memory = Memory::new();
    for block in memory.mem.iter() {
      assert_eq!(block, &0u8);
    }
  }

  #[test_case(random())]
  fn reset(value: u8) {
    let mut memory = Memory::new();
    for block in memory.mem.iter_mut() {
      *block = value;
    }
    memory.reset();
    for block in memory.mem.iter() {
      assert_eq!(block, &0u8);
    }
  }

  #[test_case(random(), random())]
  fn set(index: u16, value: u8) {
    let mut memory = Memory::new();
    memory.set(index, value);
    assert_eq!(memory.mem[index as usize], value);
  }

  #[test_case(random(), random())]
  fn get_zero_page(index: u8, value: u8) {
    let mut memory = Memory::new();
    memory.mem[index as usize] = value;
    assert_eq!(memory.get_zero_page(index), value);
  }

  #[test_case(random(), random())]
  fn get_u16(index: u16, value: u8) {
    let mut memory = Memory::new();
    memory.mem[index as usize] = value;
    assert_eq!(memory.get_u16(index), value);
  }

  #[test_case(random())]
  fn push_to_stack(value: u8) {
    let mut memory = Memory::new();
    memory.push_to_stack(value);
    assert_eq!(memory.get_u16(STACK_MAX), value);
  }

  #[test_case(random())]
  fn pop_from_stack(value: u8) {
    let mut memory = Memory::new();
    memory.set(STACK_MIN, value);
    let result = memory.pop_from_stack();
    assert_eq!(result, value);
  }
}
