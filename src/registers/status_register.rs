use log::trace;
use std::fmt::{Display, Formatter};

/// A 6502 status register with bitmasking implementation.
///
/// # Examples
///
///```
/// let mut sr = StatusRegister::new();
/// sr.set_carry_bit();
/// assert_eq!(sr.is_carry_bit_set(), true);
///```
/// Has 7 settable flags, each corresponding to the original bit ordinal
/// of the 6502. Each bit can be set, cleared, and tested for truth.
/// ```
/// let mut sr = StatusRegister::new();
/// sr.set_negative_bit();
/// sr.clear_zero_bit();
/// sr.is_overflow_bit_set();
/// ```
pub struct StatusRegister(u8);

impl StatusRegister {
  /// Constructs a new StatusRegister instance
  pub fn new() -> StatusRegister {
    StatusRegister(0)
  }

  /// Resets the status register, clearing all bits
  pub fn reset(&mut self) {
    self.0 = 0;
  }

  /// Gets the current numerical value of the register.
  pub fn get_register(&self) -> u8 {
    self.0
  }

  /// Sets the current numerical value of the register.
  pub fn set(&mut self, val: u8) {
    self.0 = val;
  }

  /// Handles setting the bitmasked flag.
  fn set_status_register(&mut self, bit: StatusBit) {
    self.0 |= BASE.pow(bit.into())
  }

  /// Handles checking the bitmask for specific flags.
  fn is_bit_set(&self, bit: StatusBit) -> bool {
    let value = bit.into();
    (self.0 & (BASE.pow(value))) >> value == 1
  }

  /// Handles clearing the bitmasked flag
  fn unset_status_register(&mut self, bit: StatusBit) {
    self.0 &= !BASE.pow(bit.into());
  }

  pub fn set_flag(&mut self, flag: StatusBit) {
    self.set_status_register(flag);
  }

  pub fn is_flag_set(&self, flag: StatusBit) -> bool {
    self.is_bit_set(flag)
  }

  pub fn clear_flag(&mut self, flag: StatusBit) {
    self.unset_status_register(flag);
  }

  /// Sets or clears the carry flag. Logs out the calling method for tracking.
  pub fn handle_c_flag(&mut self, message: &str, carry: bool) {
    match carry {
      true => {
        trace!("{} setting carry bit", message);
        self.set_flag(StatusBit::Carry);
      }
      false => {
        trace!("{} clearing carry bit", message);
        self.clear_flag(StatusBit::Carry);
      }
    }
  }

  /// Sets or clears the overflow flag. Logs out the calling method for tracking.
  pub fn handle_v_flag(&mut self, value: u8, message: &str, carry: bool) {
    if value > 0x7F {
      trace!("{} setting overflow bit", message);
      self.set_flag(StatusBit::Overflow);
    } else if value == 0x7F && carry {
      trace!("{} setting overflow bit", message);
      self.set_flag(StatusBit::Overflow);
    }
  }

  /// Sets or clears the negative flag. Logs out the calling method for tracking.
  pub fn handle_n_flag(&mut self, value: u8, message: &str) {
    if value >> 7 == 1 {
      trace!("{} setting negative bit", message);
      self.set_flag(StatusBit::Negative);
    } else {
      trace!("{} clearing negative bit", message);
      self.clear_flag(StatusBit::Negative);
    }
  }

  /// Sets or clears the zero flag. Logs out the calling method for tracking.
  pub fn handle_z_flag(&mut self, value: u8, message: &str) {
    if value == 0 {
      trace!("{} setting zero bit", message);
      self.set_flag(StatusBit::Zero);
    } else {
      trace!("{} clearing zero bit", message);
      self.clear_flag(StatusBit::Zero);
    }
  }
}

impl Display for StatusRegister {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let c = self.is_flag_set(StatusBit::Carry);
    let z = self.is_flag_set(StatusBit::Zero);
    let i = self.is_flag_set(StatusBit::Interrupt);
    let d = self.is_flag_set(StatusBit::Decimal);
    let b = self.is_flag_set(StatusBit::Break);
    let v = self.is_flag_set(StatusBit::Overflow);
    let n = self.is_flag_set(StatusBit::Negative);
    write!(
      f,
      "\n\tCarryBit: {:?}\n\tZeroBit: {:?}\n\tInterruptBit: {:?}\n\tDecimalBit: {:?}\n\tBreakBit: {:?}\n\tOverflowBit: {:?}\n\tNegativeBit: {:?}",
      c, z, i, d, b, v, n
    )
  }
}

// To ensure accuracy in our bit location we have a fake Unused bit that doesn't get used anywhere.
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum StatusBit {
  Carry,
  Zero,
  Interrupt,
  Decimal,
  Break,
  Unused,
  Overflow,
  Negative,
}

impl StatusBit {
  fn into(self) -> u32 {
    self as u32
  }
}

const BASE: u8 = 2;

#[cfg(test)]
mod tests {
  use super::*;
  use test_case::test_case;

  #[test_case(StatusBit::Carry; "Carry bit")]
  #[test_case(StatusBit::Zero; "Zero bit")]
  #[test_case(StatusBit::Overflow; "Overflow bit")]
  #[test_case(StatusBit::Negative; "Negative bit")]
  #[test_case(StatusBit::Decimal; "Decimal bit")]
  #[test_case(StatusBit::Break; "Break bit")]
  #[test_case(StatusBit::Interrupt; "Interrupt bit")]
  fn bits(bit: StatusBit) {
    let mut reg = StatusRegister::new();
    reg.set_flag(bit);
    assert_eq!(reg.0 > 1, true);
    assert_eq!(reg.is_flag_set(bit), true);
    reg.clear_flag(bit);
    assert_eq!(reg.0, 0);
    assert_eq!(reg.is_flag_set(bit), false);
  }

  #[test]
  fn set_all_bits() {
    let mut reg = StatusRegister::new();
    reg.set_flag(StatusBit::Carry);
    reg.set_flag(StatusBit::Zero);
    reg.set_flag(StatusBit::Interrupt);
    reg.set_flag(StatusBit::Break);
    reg.set_flag(StatusBit::Decimal);
    reg.set_flag(StatusBit::Overflow);
    reg.set_flag(StatusBit::Negative);
    assert_eq!(reg.is_flag_set(StatusBit::Carry), true);
    assert_eq!(reg.is_flag_set(StatusBit::Zero), true);
    assert_eq!(reg.is_flag_set(StatusBit::Interrupt), true);
    assert_eq!(reg.is_flag_set(StatusBit::Break), true);
    assert_eq!(reg.is_flag_set(StatusBit::Decimal), true);
    assert_eq!(reg.is_flag_set(StatusBit::Overflow), true);
    assert_eq!(reg.is_flag_set(StatusBit::Negative), true);
  }

  #[test]
  fn reset_bits() {
    let mut reg = StatusRegister::new();
    reg.set_flag(StatusBit::Carry);
    reg.set_flag(StatusBit::Zero);
    reg.set_flag(StatusBit::Interrupt);
    reg.set_flag(StatusBit::Break);
    reg.set_flag(StatusBit::Decimal);
    reg.set_flag(StatusBit::Overflow);
    reg.set_flag(StatusBit::Negative);
    reg.reset();
    assert_eq!(reg.0, 0);
  }

  #[test]
  fn handle_carry_set() {
    let mut reg = StatusRegister::new();
    reg.handle_c_flag("test", true);
    assert_eq!(reg.is_flag_set(StatusBit::Carry), true);
  }

  #[test]
  fn handle_carry_clear() {
    let mut reg = StatusRegister::new();
    reg.set_flag(StatusBit::Carry);
    reg.handle_c_flag("test", false);
    assert_eq!(reg.is_flag_set(StatusBit::Carry), false);
  }

  #[test]
  fn handle_overflow_set() {
    let mut reg = StatusRegister::new();
    reg.handle_v_flag(0x80, "test", false);
    assert_eq!(reg.is_flag_set(StatusBit::Overflow), true);
  }

  #[test]
  fn handle_overflow_set_carry() {
    let mut reg = StatusRegister::new();
    reg.handle_v_flag(0x7F, "test", true);
    assert_eq!(reg.is_flag_set(StatusBit::Overflow), true);
  }

  #[test]
  fn handle_zero_set() {
    let mut reg = StatusRegister::new();
    reg.handle_z_flag(0x0, "test");
    assert_eq!(reg.is_flag_set(StatusBit::Overflow), true);
  }

  #[test]
  fn handle_zero_clear() {
    let mut reg = StatusRegister::new();
    reg.set_flag(StatusBit::Zero);
    reg.handle_z_flag(0x1, "test");
    assert_eq!(reg.is_flag_set(StatusBit::Zero), false);
  }

  #[test]
  fn handle_negative_set() {
    let mut reg = StatusRegister::new();
    reg.handle_n_flag(0x80, "test");
    assert_eq!(reg.is_flag_set(StatusBit::Negative), true);
  }

  #[test]
  fn handle_negative_clear() {
    let mut reg = StatusRegister::new();
    reg.set_flag(StatusBit::Negative);
    reg.handle_n_flag(0x1, "test");
    assert_eq!(reg.is_flag_set(StatusBit::Negative), false);
  }
}
