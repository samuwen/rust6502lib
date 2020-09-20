use log::{debug, trace, warn};
/// Stack pointer works top down, so we start at 0xFF
const START_INDEX: u8 = 0xFF;

/// Emulation of a Stack Pointer. Has push and pop (pull in 6502 parlance)
/// operations, and is used as part of the memory stack. Works top down,
/// so starts at 0xFF.
///
/// The stack pointer always points at the next free value, so pushing will
/// return the current value and decrement, while popping will have to
/// increment before returning the value.
pub struct StackPointer(u8);

impl StackPointer {
  /// Creates and initializes a new stack pointer to the 0xFF index.
  pub fn new() -> StackPointer {
    debug!("Initializing new Stack Pointer");
    StackPointer(START_INDEX)
  }

  /// Resets the stack pointer to default state, the 0xFF index.
  pub fn reset(&mut self) {
    debug!("Resetting stack pointer");
    self.0 = START_INDEX;
  }

  /// Gets the current value of the stack pointer without mutating it.
  pub fn get(&self) -> u8 {
    warn!("Getting stack pointer value. Might be weird behavior");
    self.0
  }

  /// Sets the current value of the stack pointer.
  pub fn set(&mut self, val: u8) {
    warn!("Setting stack pointer value. Might be weird behavior");
    self.0 = val;
  }

  /// Handles the pointer side of pushing a value onto the stack. As the
  /// stack pointer always points to the current free value, this just
  /// returns the current value and then increments.
  ///
  /// Returns a u16 to be convenient with the memory model.
  pub fn push(&mut self) -> u16 {
    let val = self.0;
    self.0 = self.0.wrapping_sub(1);
    debug!(
      "Push to stack pointer. Pointer val: {} and return val: {}",
      self.0, val
    );
    val as u16
  }

  /// Handles the poiner side of popping (pulling) a value off of the stack.
  /// As the stack pointer always points to the current free value, this must
  /// increment before it returns a value.
  ///
  /// Returns a u16 to be convenient with the memory model.
  pub fn pop(&mut self) -> u16 {
    self.0 = self.0.wrapping_add(1);
    debug!("Pop from stack pointer. Pointer val: {}", self.0,);
    self.0 as u16
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rand::random;
  use test_case::test_case;

  #[test]
  fn new() {
    let sp = StackPointer::new();
    assert_eq!(sp.0, START_INDEX);
  }

  #[test_case(random())]
  fn reset(value: u8) {
    let mut sp = StackPointer::new();
    sp.0 = value;
    sp.reset();
    assert_eq!(sp.0, START_INDEX);
  }

  #[test_case(random())]
  fn get(value: u8) {
    let mut sp = StackPointer::new();
    sp.0 = value;
    assert_eq!(sp.get(), value);
  }

  #[test_case(random())]
  fn set(value: u8) {
    let mut sp = StackPointer::new();
    sp.set(value);
    assert_eq!(sp.0, value);
  }

  #[test_case(0x45; "No wrap")]
  #[test_case(0x00; "Wrap")]
  fn push(value: u8) {
    let mut sp = StackPointer::new();
    sp.0 = value;
    let result = sp.push();
    assert_eq!(result, value as u16);
    assert_eq!(sp.0, value.wrapping_sub(1));
  }

  #[test_case(0x45; "No wrap")]
  #[test_case(0xFF; "Wrap")]
  fn pop(value: u8) {
    let mut sp = StackPointer::new();
    sp.0 = value;
    let result = sp.pop();
    assert_eq!(result, value.wrapping_add(1) as u16);
    assert_eq!(sp.0, value.wrapping_add(1));
  }
}
