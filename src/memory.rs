use hex::{decode, encode};

pub struct Memory {
  mem: [u8; 0xFFFF],
}

impl Memory {
  pub fn new() -> Memory {
    Memory { mem: [0; 0xFFFF] }
  }

  pub fn reset(&mut self) {
    self.mem = [0; 0xFFFF];
  }

  #[allow(dead_code)]
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
    self.mem[index as usize]
  }

  pub fn get_u16_and_register(&self, index: u16, register: u8) -> u8 {
    self.mem[(index + register as u16) as usize]
  }

  /// Computes a memory address and returns the value contained within.
  ///
  /// Takes in a pair of values and gets a Most Significant and Least Significant
  /// Byte pair from them, having added the register value first.
  /// Then arranges the values in little endian and returns the index.
  pub fn get_pre_indexed_data(&self, operand: u8, register: u8) -> u8 {
    let index = self.get_pre_adjusted_index(operand, register);
    self.mem[index as usize]
  }

  pub fn get_pre_adjusted_index(&self, operand: u8, register: u8) -> u16 {
    let mut lsb = self.get_lsb(operand.wrapping_add(register));
    let msb = self.get_msb(operand.wrapping_add(register));
    lsb.push_str(&msb);
    u16::from_str_radix(&lsb, 16).unwrap()
  }

  /// Computes a memory address, adds a register value, and returns the value contained within.
  ///
  /// Takes in a pair of values and gets a Most Significant and Least Significant
  /// Byte pair from theem. Then arranges the values in little endian, adds the register value
  /// to the address, then returns the index.
  pub fn get_post_indexed_data(&self, operand: u8, register: u8) -> u8 {
    let index = self.get_post_adjusted_index(operand, register);
    self.mem[index as usize]
  }

  pub fn get_post_adjusted_index(&self, operand: u8, register: u8) -> u16 {
    let mut lsb = self.get_lsb(operand);
    let msb = self.get_msb(operand);
    lsb.push_str(&msb);
    u16::from_str_radix(&lsb, 16).unwrap() + register as u16
  }

  pub fn get_msb(&self, index: u8) -> String {
    encode(vec![self.mem[index as usize]])
  }

  pub fn get_lsb(&self, index: u8) -> String {
    encode(vec![self.mem[(index + 1) as usize]])
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
  fn get_u16_and_register() {
    let mut memory = Memory::new();
    memory.mem[0x1234] = 0x56;
    assert_eq!(memory.get_u16_and_register(0x1224, 0x10), 0x56);
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
  fn lsb() {
    let mut memory = Memory::new();
    memory.mem[0x50] = 0xFF;
    assert_eq!(memory.get_lsb(0x4F), "ff");
  }

  #[test]
  fn msb() {
    let mut memory = Memory::new();
    memory.mem[0x50] = 0xFF;
    assert_eq!(memory.get_msb(0x50), "ff");
  }
}
