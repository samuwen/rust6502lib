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
    memory.get_u16(self.get_and_increase())
  }

  /// Gets the next two values at the program counter and increases by 2
  pub fn get_two_operands(&mut self, memory: &Memory) -> [u8; 2] {
    [
      memory.get_u16(self.get_and_increase()),
      memory.get_u16(self.get_and_increase()),
    ]
  }

  pub fn increase(&mut self, amount: u8) {
    self.value += amount as u16;
  }

  pub fn decrease(&mut self, amount: u8) {
    self.value -= amount as u16;
  }

  pub fn jump(&mut self, index: u16) {
    self.value = index;
  }

  fn get_and_increase(&mut self) -> u16 {
    let v = self.value;
    self.increase(1);
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
  fn increase() {
    let mut pc = ProgramCounter::new();
    pc.increase(2);
    assert_eq!(pc.value, 2);
  }
}
