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

  pub fn decrement(&mut self) {
    self.value.wrapping_sub(2);
  }
}
