pub struct StackPointer {
  value: u8,
}

impl StackPointer {
  pub fn new() -> StackPointer {
    StackPointer { value: 0 }
  }

  pub fn reset(&mut self) {
    self.value = 0;
  }

  pub fn get(&self) -> u8 {
    self.value
  }

  #[allow(dead_code)]
  pub fn decrement(&mut self) {
    self.value = self.value.wrapping_sub(2);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new() {
    let sp = StackPointer::new();
    assert_eq!(sp.value, 0);
  }

  #[test]
  fn reset() {
    let mut sp = StackPointer::new();
    sp.value = 25;
    sp.reset();
    assert_eq!(sp.value, 0);
  }

  #[test]
  fn get() {
    let mut sp = StackPointer::new();
    sp.value = 39;
    assert_eq!(sp.get(), 39);
  }

  #[test]
  fn decrement() {
    let mut sp = StackPointer::new();
    sp.decrement();
    assert_eq!(sp.value, 254);
  }
}
