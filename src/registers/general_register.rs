pub struct GeneralRegister {
  value: u8,
}

impl GeneralRegister {
  pub fn new() -> GeneralRegister {
    GeneralRegister { value: 0 }
  }

  pub fn reset(&mut self) {
    self.value = 0;
  }

  pub fn get(&self) -> u8 {
    self.value
  }

  pub fn set(&mut self, v: u8) {
    self.value = v;
  }
}
