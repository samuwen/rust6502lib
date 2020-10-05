pub fn is_letter(c: &char) -> bool {
  c.is_alphabetic()
}

pub fn is_operator(c: &char) -> bool {
  match c {
    '=' | ':' | '<' | '>' | '#' | ',' | '(' | ')' => true,
    _ => false,
  }
}

pub fn is_number(c: &char) -> bool {
  match c {
    '$' | '%' => true,
    _ => c.is_digit(10),
  }
}

pub fn is_string(c: &char) -> bool {
  match c {
    '"' | '\'' => true,
    _ => false,
  }
}

pub fn is_decimal(c: &char) -> bool {
  match c {
    '.' => true,
    _ => false,
  }
}
