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

  pub fn increase(&mut self, amount: u8) {
    self.value += amount as u16;
  }

  pub fn decrease(&mut self, amount: u8) {
    self.value -= amount as u16;
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
