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

  pub fn get(&self, index: usize) -> u8 {
    self.mem[index]
  }

  pub fn get_u16(&self, index: u16) -> u8 {
    self.get(index as usize)
  }

  /// Computes a memory address and returns the value contained within.
  ///
  /// Takes in a pair of values and gets a Most Significant and Least Significant
  /// Byte pair from them, having added the register value first.
  /// Then arranges the values in little endian and returns the index.
  pub fn get_pre_indexed_data(&self, operand: u8) -> u8 {
    let lsb = self.get_zero_page(operand);
    let msb = self.get_zero_page(operand + 1);
    let index = self.get_u16_index(lsb, msb);
    self.mem[index as usize]
  }

  /// Computes a memory address, adds a register value, and returns the value contained within.
  ///
  /// Takes in a pair of values and gets a Most Significant and Least Significant
  /// Byte pair from theem. Then arranges the values in little endian, adds the register value
  /// to the address, then returns the index.
  pub fn get_post_indexed_data(&self, operand: u8, register_value: u8) -> u8 {
    let lsb = self.get_zero_page(operand);
    let msb = self.get_zero_page(operand + 1);
    let index = self.get_u16_index(lsb, msb);
    self.mem[(index + register_value as u16) as usize]
  }

  pub fn get_u16_index(&self, lsb: u8, msb: u8) -> u16 {
    u16::from_le_bytes([lsb, msb])
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
    assert_eq!(memory.get_pre_indexed_data(0x98), 0x56);
  }

  #[test]
  fn get_post_indexed_data() {
    let mut memory = Memory::new();
    memory.mem[0x86] = 0x28;
    memory.mem[0x87] = 0x40;
    memory.mem[0x4038] = 0x56;
    assert_eq!(memory.get_post_indexed_data(0x86, 0x10), 0x56);
  }
}
