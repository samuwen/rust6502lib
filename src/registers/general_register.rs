use log::{debug, trace};
/// A generic 6502 register, X Y or A
pub struct GeneralRegister(u8);

impl GeneralRegister {
  /// Creates a new register
  pub fn new() -> GeneralRegister {
    debug!("Initializing a general register");
    GeneralRegister(0)
  }

  ///  Rests the register value to 0
  pub fn reset(&mut self) {
    debug!("Resetting a general register");
    self.0 = 0;
  }

  /// Gets the register's value
  pub fn get(&self) -> u8 {
    trace!("Getting register value: {}", self.0);
    self.0
  }

  /// Sets the register's value
  pub fn set(&mut self, v: u8) {
    trace!("Setting register value to: {}", v);
    self.0 = v;
  }

  /// Increment's the register's value by 1. Wraps if overflow.
  pub fn increment(&mut self) {
    self.0 = self.0.wrapping_add(1);
  }

  /// Decrement's the register's value by 1. Wraps if overflow.
  pub fn decrement(&mut self) {
    self.0 = self.0.wrapping_sub(1);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_case::test_case;

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

  #[test_case(0, 1; "No wrap")]
  #[test_case(0xFF, 0; "Wrap")]
  fn increment(start: u8, end: u8) {
    let mut gr = GeneralRegister::new();
    gr.set(start);
    gr.increment();
    assert_eq!(gr.0, end);
  }

  #[test_case(1, 0; "No wrap")]
  #[test_case(0, 0xFF; "Wrap")]
  fn decrement(start: u8, end: u8) {
    let mut gr = GeneralRegister::new();
    gr.set(start);
    gr.decrement();
    assert_eq!(gr.0, end);
  }
}
