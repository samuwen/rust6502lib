use crate::STARTING_MEMORY_BLOCK;
use log::{debug, trace};

/// The Program Counter for the computer. Keeps track of where the computer's
/// execution is transpiring.
pub struct ProgramCounter {
  value: u16,
  start: u16,
}

impl ProgramCounter {
  /// Creates and initializes a new PC. Initializes the initial value to the
  /// default memory block. If you want to initialize at another location,
  /// use new_at().
  pub fn new() -> ProgramCounter {
    debug!(
      "Initializing a new Program Counter at: {}",
      STARTING_MEMORY_BLOCK
    );
    ProgramCounter {
      value: STARTING_MEMORY_BLOCK,
      start: STARTING_MEMORY_BLOCK,
    }
  }

  /// Creates and initializes a new PC. Initializes to the provided value.
  /// Warning: You can potentially initialize to values that prevent your
  /// system from working.
  #[allow(dead_code)]
  pub fn new_at(block: u16) -> ProgramCounter {
    debug!("Initializing a new Program Counter at: {}", block);
    ProgramCounter {
      value: block,
      start: block,
    }
  }

  /// Resets the PC to the original start block
  pub fn reset(&mut self) {
    debug!("Resetting Program Counter to: {}", self.start);
    self.value = self.start;
  }

  /// Gets the current state of the program counter. Does not mutate.
  pub fn get(&self) -> usize {
    self.value as usize
  }

  /// Adds the specified value to the program counter, wrapping if overflow.
  /// Tests if the addition crossed a page boundary and returns true if it did.
  pub fn increase(&mut self, amount: u8) -> bool {
    let did_cross = self.test_page_boundary_add(amount);
    self.value = self.value.wrapping_add(amount as u16);
    did_cross
  }

  /// Subtracts the specified value to the program counter, wrapping if overflow.
  /// Tests if the subtraction crossed a page boundary and returns true if it did.
  pub fn decrease(&mut self, amount: u8) -> bool {
    let did_cross = self.test_page_boundary_sub(amount);
    self.value = self.value.wrapping_sub(amount as u16);
    did_cross
  }

  /// Tests if an addition would cross a page boundary and returns true if it would.
  fn test_page_boundary_add(&mut self, amount: u8) -> bool {
    let ops = self.value.to_le_bytes();
    let crossed = ops[0].overflowing_add(amount).1;
    trace!("Page boundary crossing test result: {}", crossed);
    crossed
  }

  /// Tests if a subtraction would cross a page boundary and returns true if it would.
  fn test_page_boundary_sub(&mut self, amount: u8) -> bool {
    let ops = self.value.to_le_bytes();
    let crossed = ops[0].overflowing_sub(amount).1;
    trace!("Page boundary crossing test result: {}", crossed);
    crossed
  }

  /// Sets the program counter to a new location to proceed execution from there.
  pub fn jump(&mut self, index: u16) {
    self.value = index;
  }

  /// Increments the PC and then returns the new value. Used primarily for retrieving
  /// operands in a manner that preserves processor sync.
  pub fn get_and_increase(&mut self) -> u16 {
    self.increase(1);
    self.value
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rand::random;
  use test_case::test_case;

  #[test]
  fn new() {
    let pc = ProgramCounter::new();
    assert_eq!(pc.value, STARTING_MEMORY_BLOCK);
    assert_eq!(pc.start, STARTING_MEMORY_BLOCK);
  }

  #[test_case(random(); "New at with random value")]
  fn new_at(value: u16) {
    let pc = ProgramCounter::new_at(value);
    assert_eq!(pc.value, value);
    assert_eq!(pc.start, value);
  }

  #[test]
  fn reset() {
    let mut pc = ProgramCounter::new();
    pc.value = random();
    pc.reset();
    assert_eq!(pc.value, pc.start);
  }

  #[test_case(random(); "Get value")]
  fn get(value: u16) {
    let mut pc = ProgramCounter::new();
    pc.value = value;
    assert_eq!(pc.get(), value as usize);
  }

  #[test_case(0x1234, 0x56; "Increase no wrap")]
  #[test_case(0xFFFE, 0xFF; "Increase wrap")]
  fn increase(start: u16, increase: u8) {
    let mut pc = ProgramCounter::new_at(start);
    pc.increase(increase);
    let result = start.wrapping_add(increase as u16);
    assert_eq!(pc.value, result);
  }

  #[test_case(0x1234, 0x56; "Decrease no wrap")]
  #[test_case(0x0002, 0xFF; "Decrease wrap")]
  fn decrease(start: u16, decrease: u8) {
    let mut pc = ProgramCounter::new_at(start);
    pc.decrease(decrease);
    let result = start.wrapping_sub(decrease as u16);
    assert_eq!(pc.value, result);
  }

  #[test_case(0x1212, 0x34, false; "Boundary test no cross")]
  #[test_case(0x12FF, 0xFF, true; "Boundary test with cross")]
  fn test_page_boundary_add(start: u16, increase: u8, result: bool) {
    let mut pc = ProgramCounter::new_at(start);
    let did_cross = pc.test_page_boundary_add(increase);
    assert_eq!(did_cross, result);
  }

  #[test_case(0x12FF, 0x34, false; "Boundary test no cross")]
  #[test_case(0x1212, 0xFF, true; "Boundary test with cross")]
  fn test_page_boundary_sub(start: u16, decrease: u8, result: bool) {
    let mut pc = ProgramCounter::new_at(start);
    let did_cross = pc.test_page_boundary_sub(decrease);
    assert_eq!(did_cross, result);
  }

  #[test_case(random(); "Random jump")]
  fn jump(amt: u16) {
    let mut pc = ProgramCounter::new();
    pc.jump(amt);
    assert_eq!(pc.value, amt);
  }

  #[test]
  fn get_and_increase() {
    let mut pc = ProgramCounter::new();
    let op = pc.get_and_increase();
    assert_eq!(op, STARTING_MEMORY_BLOCK + 1);
  }
}
