use crate::memory::Memory;

pub struct ProgramCounter {
  value: u16,
}

impl ProgramCounter {
  pub fn new() -> ProgramCounter {
    ProgramCounter { value: 0 }
  }

  pub fn reset(&mut self) {
    self.value = 0;
  }

  /// Gets the current state of the program counter. Does not mutate.
  pub fn get(&self) -> usize {
    self.value as usize
  }

  /// Gets the value at the program counter then advances by 1
  pub fn get_single_operand(&mut self, memory: &Memory) -> u8 {
    memory.get_u16(self.get_and_advance())
  }

  /// Gets the next two values at the program counter and advances by 2
  pub fn get_two_operands(&mut self, memory: &Memory) -> [u8; 2] {
    [
      memory.get_u16(self.get_and_advance()),
      memory.get_u16(self.get_and_advance()),
    ]
  }

  pub fn advance(&mut self, amount: u16) {
    self.value += amount;
  }

  fn get_and_advance(&mut self) -> u16 {
    let v = self.value;
    self.advance(1);
    v
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new() {
    let pc = ProgramCounter::new();
    assert_eq!(pc.value, 0);
  }

  #[test]
  fn reset() {
    let mut pc = ProgramCounter::new();
    pc.value = 25;
    pc.reset();
    assert_eq!(pc.value, 0);
  }

  #[test]
  fn get() {
    let mut pc = ProgramCounter::new();
    pc.value = 3990;
    assert_eq!(pc.get(), 3990);
  }

  #[test]
  fn advance() {
    let mut pc = ProgramCounter::new();
    pc.advance(2);
    assert_eq!(pc.value, 2);
  }
}
