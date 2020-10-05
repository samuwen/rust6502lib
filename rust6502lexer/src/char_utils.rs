/// Determines if the character is a letter
pub fn is_letter(c: &char) -> bool {
  c.is_alphabetic()
}

/// Determines if the character is an operator
pub fn is_operator(c: &char) -> bool {
  match c {
    '=' | ':' | '<' | '>' | '#' | ',' | '(' | ')' => true,
    _ => false,
  }
}

/// Determines if the character is a number
pub fn is_number(c: &char) -> bool {
  match c {
    '$' | '%' => true,
    _ => c.is_digit(10),
  }
}

/// Determines if the character is a quote mark, indicating a string
pub fn is_string(c: &char) -> bool {
  match c {
    '"' | '\'' => true,
    _ => false,
  }
}

/// Determines if the character is a decimal point
pub fn is_decimal(c: &char) -> bool {
  match c {
    '.' => true,
    _ => false,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn letter() {
    assert_eq!(is_letter(&'a'), true);
    assert_eq!(is_letter(&'0'), false);
  }

  #[test]
  fn number() {
    assert_eq!(is_number(&'a'), false);
    assert_eq!(is_number(&'0'), true);
  }

  #[test]
  fn operator() {
    assert_eq!(is_operator(&'='), true);
    assert_eq!(is_operator(&'('), true);
    assert_eq!(is_operator(&'0'), false);
    assert_eq!(is_operator(&'a'), false);
  }

  #[test]
  fn string() {
    assert_eq!(is_string(&'\"'), true);
    assert_eq!(is_string(&'a'), false);
  }

  #[test]
  fn decimal() {
    assert_eq!(is_decimal(&'.'), true);
    assert_eq!(is_decimal(&'a'), false);
  }
}
