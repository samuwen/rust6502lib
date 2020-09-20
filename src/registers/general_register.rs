pub struct GeneralRegister(u8);

impl GeneralRegister {
  pub fn new() -> GeneralRegister {
    GeneralRegister(0)
  }

  pub fn reset(&mut self) {
    self.0 = 0;
  }

  pub fn get(&self) -> u8 {
    self.0
  }

  pub fn set(&mut self, v: u8) {
    self.0 = v;
  }

  pub fn increment(&mut self) {
    self.0 = self.0.wrapping_add(1);
  }

  pub fn decrement(&mut self) {
    self.0 = self.0.wrapping_sub(1);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new() {
    let gr = GeneralRegister::new();
    assert_eq!(gr.0, 0);
  }

  #[test]
  fn reset() {
    let mut gr = GeneralRegister::new();
    gr.0 = 25;
    gr.reset();
    assert_eq!(gr.0, 0);
  }

  #[test]
  fn get() {
    let mut gr = GeneralRegister::new();
    gr.0 = 39;
    assert_eq!(gr.get(), 39);
  }

  #[test]
  fn set() {
    let mut gr = GeneralRegister::new();
    gr.set(2);
    assert_eq!(gr.0, 2);
  }
}
