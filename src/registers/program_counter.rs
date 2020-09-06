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

  pub fn get(&self) -> u16 {
    self.value
  }

  pub fn increment(&mut self) {
    self.value += 2;
  }
}
