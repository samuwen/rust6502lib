pub struct StackPointer {
  value: u8,
}

impl StackPointer {
  pub fn new() -> StackPointer {
    StackPointer { value: 0xFF }
  }

  pub fn reset(&mut self) {
    self.value = 0xFF;
  }

  pub fn get(&self) -> u8 {
    self.value
  }

  pub fn set(&mut self, val: u8) {
    self.value = val;
  }

  pub fn push(&mut self) -> u16 {
    let val = self.value;
    self.value = self.value.wrapping_sub(1);
    val as u16
  }

  pub fn pop(&mut self) -> u16 {
    let val = self.value;
    self.value = self.value.wrapping_add(1);
    val as u16
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new() {
    let sp = StackPointer::new();
    assert_eq!(sp.value, 0xFF);
  }

  #[test]
  fn reset() {
    let mut sp = StackPointer::new();
    sp.value = 25;
    sp.reset();
    assert_eq!(sp.value, 0xFF);
  }

  #[test]
  fn get() {
    let mut sp = StackPointer::new();
    sp.value = 39;
    assert_eq!(sp.get(), 39);
  }

  #[test]
  fn push() {
    let mut sp = StackPointer::new();
    sp.value = 0x45;
    let result = sp.push();
    assert_eq!(result, 0x45);
    assert_eq!(sp.value, 0x44);
  }

  #[test]
  fn pop() {
    let mut sp = StackPointer::new();
    sp.value = 0x45;
    let result = sp.pop();
    assert_eq!(result, 0x45);
    assert_eq!(sp.value, 0x46);
  }
}
