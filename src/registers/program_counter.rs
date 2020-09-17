use crate::STARTING_MEMORY_BLOCK;

pub struct ProgramCounter {
  value: u16,
}

impl ProgramCounter {
  pub fn new() -> ProgramCounter {
    ProgramCounter {
      value: STARTING_MEMORY_BLOCK,
    }
  }

  pub fn reset(&mut self) {
    self.value = STARTING_MEMORY_BLOCK;
  }

  /// Gets the current state of the program counter. Does not mutate.
  pub fn get(&self) -> usize {
    self.value as usize
  }

  pub fn to_le_bytes(&self) -> [u8; 2] {
    self.value.to_le_bytes()
  }

  pub fn increment(&mut self) {
    self.value.wrapping_add(1);
  }

  pub fn increase(&mut self, amount: u8) -> bool {
    self.value = self.value.wrapping_add(amount as u16);
    self.test_page_boundary_add(amount)
  }

  pub fn decrease(&mut self, amount: u8) -> bool {
    self.value = self.value.wrapping_sub(amount as u16);
    self.test_page_boundary_sub(amount)
  }

  fn test_page_boundary_add(&mut self, amount: u8) -> bool {
    let ops = self.value.to_le_bytes();
    return ops[0].overflowing_add(amount).1;
  }

  fn test_page_boundary_sub(&mut self, amount: u8) -> bool {
    let ops = self.value.to_le_bytes();
    return ops[0].overflowing_sub(amount).1;
  }

  pub fn jump(&mut self, index: u16) {
    self.value = index;
  }

  /// Increments the PC and then returns the new value.
  pub fn get_and_increase(&mut self) -> u16 {
    self.increase(1);
    self.value
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
