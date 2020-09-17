use crate::StackPointer;
use log::warn;

pub struct Memory {
  mem: [u8; 0xFFFF],
  sp: StackPointer,
}

impl Memory {
  pub fn new() -> Memory {
    Memory {
      mem: [0; 0xFFFF],
      sp: StackPointer::new(),
    }
  }

  pub fn reset(&mut self) {
    self.mem = [0; 0xFFFF];
    self.sp.reset();
  }

  pub fn set(&mut self, index: u16, value: u8) {
    self.mem[index as usize] = value;
  }

  pub fn set_zero_page(&mut self, index: u8, value: u8) {
    self.mem[index as usize] = value;
  }

  pub fn get_zero_page(&self, index: u8) -> u8 {
    self.mem[index as usize]
  }

  pub fn get_u16(&self, index: u16) -> u8 {
    if index <= 0x1FF && index >= 0x100 {
      warn!("Accessing memory from the stack improperly!");
    }
    self.mem[index as usize]
  }

  /// Adds a value to the stack. Takes in a value to be entered and the stack pointer.
  pub fn push_to_stack(&mut self, value: u8) {
    self.mem[(0x100 | self.sp.push()) as usize] = value;
  }

  /// Takes a value from the stack. Returns the value at the current stack pointer.
  pub fn pop_from_stack(&mut self) -> u8 {
    self.mem[(0x100 | self.sp.pop()) as usize]
  }

  pub fn get_pre_adjusted_index(&self, operand: u8, register: u8) -> u16 {
    u16::from_le_bytes([
      self.get_zero_page(operand.wrapping_add(register)),
      self.get_zero_page(operand.wrapping_add(register).wrapping_add(1)),
    ])
  }

  pub fn get_post_adjusted_index(&self, operand: u8, register: u8) -> u16 {
    let unadjusted_index = u16::from_le_bytes([
      self.get_zero_page(operand),
      self.get_zero_page(operand.wrapping_add(1)),
    ]);
    unadjusted_index.wrapping_add(register as u16)
  }

  pub fn get_stack_pointer(&self) -> &StackPointer {
    &self.sp
  }

  pub fn set_stack_pointer(&mut self, val: u8) {
    self.sp.set(val);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new() {
    let memory = Memory::new();
    for block in memory.mem.iter() {
      assert_eq!(block, &0u8);
    }
  }

  #[test]
  fn reset() {
    let mut memory = Memory::new();
    for block in memory.mem.iter_mut() {
      *block = 0x12;
    }
    memory.reset();
    for block in memory.mem.iter() {
      assert_eq!(block, &0u8);
    }
  }

  #[test]
  fn set() {
    let mut memory = Memory::new();
    memory.set(0x12, 0x34);
    assert_eq!(memory.mem[0x12], 0x34);
  }

  #[test]
  fn get_u8() {
    let mut memory = Memory::new();
    memory.mem[0x12] = 0x34;
    assert_eq!(memory.get_zero_page(0x12), 0x34);
  }

  #[test]
  fn get_u16() {
    let mut memory = Memory::new();
    memory.mem[0x1234] = 0x56;
    assert_eq!(memory.get_u16(0x1234), 0x56);
  }

  #[test]
  fn get_pre_indexed_data() {
    let mut memory = Memory::new();
    memory.mem[0x98] = 0x34;
    memory.mem[0x99] = 0x12;
    memory.mem[0x1234] = 0x56;
    assert_eq!(memory.get_pre_indexed_data(0x88, 0x10), 0x56);
  }

  #[test]
  fn get_pre_adjusted_index() {
    let mut memory = Memory::new();
    memory.mem[0x98] = 0x34;
    memory.mem[0x99] = 0x12;
    assert_eq!(memory.get_pre_adjusted_index(0x88, 0x10), 0x1234);
  }

  #[test]
  fn get_post_indexed_data() {
    let mut memory = Memory::new();
    memory.mem[0x86] = 0x28;
    memory.mem[0x87] = 0x40;
    memory.mem[0x4038] = 0x56;
    assert_eq!(memory.get_post_indexed_data(0x86, 0x10), 0x56);
  }

  #[test]
  fn get_post_adjusted_index() {
    let mut memory = Memory::new();
    memory.mem[0x86] = 0x28;
    memory.mem[0x87] = 0x40;
    assert_eq!(memory.get_post_adjusted_index(0x86, 0x10), 0x4038);
  }

  #[test]
  fn push_to_stack() {
    let mut memory = Memory::new();
    memory.push_to_stack(0x10);
    assert_eq!(memory.get_u16(0x1FF), 0x10);
  }

  #[test]
  fn pop_from_stack() {
    let mut memory = Memory::new();
    memory.set(0x1FF, 0x10);
    let result = memory.pop_from_stack();
    assert_eq!(result, 0x10);
  }
}
