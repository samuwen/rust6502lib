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

  fn set_status_register(&mut self, bit: StatusBit) {
    self.register |= BASE.pow(bit.into())
  }

  fn is_bit_set(&self, bit: StatusBit) -> bool {
    (self.register >> bit.into()) == 1
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

  pub fn set_break_bit(&mut self) {
    self.set_status_register(StatusBit::Break);
  }

  pub fn is_break_bit_set(&self) -> bool {
    self.is_bit_set(StatusBit::Break)
  }

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
}

enum StatusBit {
  Carry,
  Zero,
  Interrupt,
  Decimal,
  Break,
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
    assert_eq!(reg.register, 32);
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
    reg.register = 32;
    reg.clear_overflow_bit();
    assert_eq!(reg.register, 0);
  }

  #[test]
  fn set_negative_bit() {
    let mut reg = StatusRegister::new();
    reg.set_negative_bit();
    assert_eq!(reg.register, 64);
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
    reg.register = 64;
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
    assert_eq!(reg.register, 127);
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
}
