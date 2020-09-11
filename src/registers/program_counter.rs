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

  pub fn get(&self) -> usize {
    self.value as usize
  }

  pub fn set(&mut self, value: u16) {
    self.value = value;
  }

  pub fn get_next(&mut self) -> usize {
    self.advance(1);
    self.value as usize
  }

  pub fn advance(&mut self, amount: u16) {
    self.value += amount;
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
