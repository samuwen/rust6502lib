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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new() {
    let gr = GeneralRegister::new();
    assert_eq!(gr.value, 0);
  }

  #[test]
  fn reset() {
    let mut gr = GeneralRegister::new();
    gr.value = 25;
    gr.reset();
    assert_eq!(gr.value, 0);
  }

  #[test]
  fn get() {
    let mut gr = GeneralRegister::new();
    gr.value = 39;
    assert_eq!(gr.get(), 39);
  }

  #[test]
  fn set() {
    let mut gr = GeneralRegister::new();
    gr.set(2);
    assert_eq!(gr.value, 2);
  }
}
