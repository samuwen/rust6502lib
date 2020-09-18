use log::trace;
use std::fmt::{Display, Formatter};

pub struct StatusRegister {
  register: u8,
}

impl StatusRegister {
  pub fn new() -> StatusRegister {
    StatusRegister { register: 0 }
  }

  pub fn reset(&mut self) {
    self.register = 0;
  }

  pub fn get_register(&self) -> u8 {
    self.register
  }

  pub fn set(&mut self, val: u8) {
    self.register = val;
  }

  fn set_status_register(&mut self, bit: StatusBit) {
    self.register |= BASE.pow(bit.into())
  }

  fn is_bit_set(&self, bit: StatusBit) -> bool {
    let value = bit.into();
    (self.register & (BASE.pow(value))) >> value == 1
  }

  fn unset_status_register(&mut self, bit: StatusBit) {
    self.register &= !BASE.pow(bit.into());
  }

  pub fn set_carry_bit(&mut self) {
    self.set_status_register(StatusBit::Carry);
  }

  pub fn is_carry_bit_set(&self) -> bool {
    self.is_bit_set(StatusBit::Carry)
  }

  pub fn clear_carry_bit(&mut self) {
    self.unset_status_register(StatusBit::Carry);
  }

  pub fn set_zero_bit(&mut self) {
    self.set_status_register(StatusBit::Zero);
  }

  pub fn is_zero_bit_set(&self) -> bool {
    self.is_bit_set(StatusBit::Zero)
  }

  pub fn clear_zero_bit(&mut self) {
    self.unset_status_register(StatusBit::Zero);
  }

  pub fn set_interrupt_bit(&mut self) {
    self.set_status_register(StatusBit::Interrupt);
  }

  pub fn is_interrupt_bit_set(&self) -> bool {
    self.is_bit_set(StatusBit::Interrupt)
  }

  pub fn clear_interrupt_bit(&mut self) {
    self.unset_status_register(StatusBit::Interrupt);
  }

  #[allow(dead_code)]
  pub fn set_break_bit(&mut self) {
    self.set_status_register(StatusBit::Break);
  }

  pub fn is_break_bit_set(&self) -> bool {
    self.is_bit_set(StatusBit::Break)
  }

  #[allow(dead_code)]
  pub fn clear_break_bit(&mut self) {
    self.unset_status_register(StatusBit::Break);
  }

  pub fn set_decimal_bit(&mut self) {
    self.set_status_register(StatusBit::Decimal);
  }

  pub fn is_decimal_bit_set(&self) -> bool {
    self.is_bit_set(StatusBit::Decimal)
  }

  pub fn clear_decimal_bit(&mut self) {
    self.unset_status_register(StatusBit::Decimal);
  }

  pub fn set_overflow_bit(&mut self) {
    self.set_status_register(StatusBit::Overflow);
  }

  pub fn is_overflow_bit_set(&self) -> bool {
    self.is_bit_set(StatusBit::Overflow)
  }

  pub fn clear_overflow_bit(&mut self) {
    self.unset_status_register(StatusBit::Overflow);
  }

  pub fn set_negative_bit(&mut self) {
    self.set_status_register(StatusBit::Negative);
  }

  pub fn is_negative_bit_set(&self) -> bool {
    self.is_bit_set(StatusBit::Negative)
  }

  pub fn clear_negative_bit(&mut self) {
    self.unset_status_register(StatusBit::Negative);
  }

  pub fn handle_c_flag(&mut self, message: &str, carry: bool) {
    if carry {
      trace!("{} setting carry bit", message);
      self.set_carry_bit();
    } else {
      trace!("{} clearing carry bit", message);
      self.clear_carry_bit();
    }
  }

  pub fn handle_v_flag(&mut self, value: u8, message: &str, carry: bool) {
    if value > 0x7F {
      trace!("{} setting overflow bit", message);
      self.set_overflow_bit();
    } else if value == 0x7F && carry {
      trace!("{} setting overflow bit", message);
      self.set_overflow_bit();
    }
  }

  pub fn handle_n_flag(&mut self, value: u8, message: &str) {
    if value >> 7 == 1 {
      trace!("{} setting negative bit", message);
      self.set_negative_bit();
    } else {
      trace!("{} clearing negative bit", message);
      self.clear_negative_bit();
    }
  }

  pub fn handle_z_flag(&mut self, value: u8, message: &str) {
    if value == 0 {
      trace!("{} setting zero bit", message);
      self.set_zero_bit();
    } else {
      trace!("{} clearing zero bit", message);
      self.clear_zero_bit();
    }
  }
}

impl Display for StatusRegister {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "\n\tCarryBit: {:?}\n\tZeroBit: {:?}\n\tInterruptBit: {:?}\n\tDecimalBit: {:?}\n\tBreakBit: {:?}\n\tOverflowBit: {:?}\n\tNegativeBit: {:?}",
      self.is_carry_bit_set(), self.is_zero_bit_set(), self.is_interrupt_bit_set(), self.is_decimal_bit_set(), self.is_break_bit_set(), self.is_overflow_bit_set(), self.is_negative_bit_set()
    )
  }
}

#[allow(dead_code)]
enum StatusBit {
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

  #[test]
  fn set_carry_bit() {
    let mut reg = StatusRegister::new();
    reg.set_carry_bit();
    assert_eq!(reg.register, 1);
  }

  #[test]
  fn is_carry_set() {
    let mut reg = StatusRegister::new();
    reg.set_carry_bit();
    assert_eq!(reg.is_carry_bit_set(), true);
  }

  #[test]
  fn unset_carry_bit() {
    let mut reg = StatusRegister::new();
    reg.register = 1;
    reg.clear_carry_bit();
    assert_eq!(reg.register, 0);
  }

  #[test]
  fn set_zero_bit() {
    let mut reg = StatusRegister::new();
    reg.set_zero_bit();
    assert_eq!(reg.register, 2);
  }

  #[test]
  fn is_zero_set() {
    let mut reg = StatusRegister::new();
    reg.set_zero_bit();
    assert_eq!(reg.is_zero_bit_set(), true);
  }

  #[test]
  fn unset_zero_bit() {
    let mut reg = StatusRegister::new();
    reg.register = 2;
    reg.clear_zero_bit();
    assert_eq!(reg.register, 0);
  }

  #[test]
  fn set_interrupt_bit() {
    let mut reg = StatusRegister::new();
    reg.set_interrupt_bit();
    assert_eq!(reg.register, 4);
  }

  #[test]
  fn is_interrupt_set() {
    let mut reg = StatusRegister::new();
    reg.set_interrupt_bit();
    assert_eq!(reg.is_interrupt_bit_set(), true);
  }

  #[test]
  fn unset_interrupt_bit() {
    let mut reg = StatusRegister::new();
    reg.register = 4;
    reg.clear_interrupt_bit();
    assert_eq!(reg.register, 0);
  }

  #[test]
  fn set_decimal_bit() {
    let mut reg = StatusRegister::new();
    reg.set_decimal_bit();
    assert_eq!(reg.register, 8);
  }

  #[test]
  fn is_decimal_bit_set() {
    let mut reg = StatusRegister::new();
    reg.set_decimal_bit();
    assert_eq!(reg.is_decimal_bit_set(), true);
  }

  #[test]
  fn unset_decimal_but() {
    let mut reg = StatusRegister::new();
    reg.register = 8;
    reg.clear_decimal_bit();
    assert_eq!(reg.register, 0);
  }

  #[test]
  fn set_break_bit() {
    let mut reg = StatusRegister::new();
    reg.set_break_bit();
    assert_eq!(reg.register, 16);
  }

  #[test]
  fn is_break_bit_set() {
    let mut reg = StatusRegister::new();
    reg.set_break_bit();
    assert_eq!(reg.is_break_bit_set(), true);
  }

  #[test]
  fn unset_break_but() {
    let mut reg = StatusRegister::new();
    reg.register = 16;
    reg.clear_break_bit();
    assert_eq!(reg.register, 0);
  }

  #[test]
  fn set_overflow_bit() {
    let mut reg = StatusRegister::new();
    reg.set_overflow_bit();
    assert_eq!(reg.register, 64);
  }

  #[test]
  fn is_overflow_bit_set() {
    let mut reg = StatusRegister::new();
    reg.set_overflow_bit();
    assert_eq!(reg.is_overflow_bit_set(), true);
  }

  #[test]
  fn unset_overflow_but() {
    let mut reg = StatusRegister::new();
    reg.register = 64;
    reg.clear_overflow_bit();
    assert_eq!(reg.register, 0);
  }

  #[test]
  fn set_negative_bit() {
    let mut reg = StatusRegister::new();
    reg.set_negative_bit();
    assert_eq!(reg.register, 128);
  }

  #[test]
  fn is_negative_bit_set() {
    let mut reg = StatusRegister::new();
    reg.set_negative_bit();
    assert_eq!(reg.is_negative_bit_set(), true);
  }

  #[test]
  fn unset_negative_but() {
    let mut reg = StatusRegister::new();
    reg.register = 128;
    reg.clear_negative_bit();
    assert_eq!(reg.register, 0);
  }

  #[test]
  fn set_all_bits() {
    let mut reg = StatusRegister::new();
    reg.set_carry_bit();
    reg.set_break_bit();
    reg.set_decimal_bit();
    reg.set_interrupt_bit();
    reg.set_overflow_bit();
    reg.set_negative_bit();
    reg.set_zero_bit();
    assert_eq!(reg.is_carry_bit_set(), true);
    assert_eq!(reg.is_zero_bit_set(), true);
    assert_eq!(reg.is_break_bit_set(), true);
    assert_eq!(reg.is_decimal_bit_set(), true);
    assert_eq!(reg.is_interrupt_bit_set(), true);
    assert_eq!(reg.is_overflow_bit_set(), true);
    assert_eq!(reg.is_negative_bit_set(), true);
  }

  #[test]
  fn reset_bits() {
    let mut reg = StatusRegister::new();
    reg.set_carry_bit();
    reg.set_break_bit();
    reg.set_decimal_bit();
    reg.set_interrupt_bit();
    reg.set_overflow_bit();
    reg.set_negative_bit();
    reg.set_zero_bit();
    reg.reset();
    assert_eq!(reg.register, 0);
  }

  #[test]
  fn handle_carry_set() {
    let mut reg = StatusRegister::new();
    reg.handle_c_flag("test", true);
    assert_eq!(reg.is_carry_bit_set(), true);
  }

  #[test]
  fn handle_carry_clear() {
    let mut reg = StatusRegister::new();
    reg.set_carry_bit();
    reg.handle_c_flag("test", false);
    assert_eq!(reg.is_carry_bit_set(), false);
  }

  #[test]
  fn handle_overflow_set() {
    let mut reg = StatusRegister::new();
    reg.handle_v_flag(0x80, "test", false);
    assert_eq!(reg.is_overflow_bit_set(), true);
  }

  #[test]
  fn handle_overflow_set_carry() {
    let mut reg = StatusRegister::new();
    reg.handle_v_flag(0x7F, "test", true);
    assert_eq!(reg.is_overflow_bit_set(), true);
  }

  #[test]
  fn handle_zero_set() {
    let mut reg = StatusRegister::new();
    reg.handle_z_flag(0x0, "test");
    assert_eq!(reg.is_zero_bit_set(), true);
  }

  #[test]
  fn handle_zero_clear() {
    let mut reg = StatusRegister::new();
    reg.set_zero_bit();
    reg.handle_z_flag(0x1, "test");
    assert_eq!(reg.is_zero_bit_set(), false);
  }

  #[test]
  fn handle_negative_set() {
    let mut reg = StatusRegister::new();
    reg.handle_n_flag(0x80, "test");
    assert_eq!(reg.is_negative_bit_set(), true);
  }

  #[test]
  fn handle_negative_clear() {
    let mut reg = StatusRegister::new();
    reg.set_negative_bit();
    reg.handle_n_flag(0x1, "test");
    assert_eq!(reg.is_negative_bit_set(), false);
  }
}
