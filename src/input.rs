/// Class of str-based inputs; allowing to get a &str and advancing it by a given number of characters.
pub trait StrBased: PartialEq {
  fn as_str(&self) -> &str;
  fn advance(self, count: usize) -> Self;
}

/// Class of column-based inputs; i.e. accepting the concept of a “column.”
pub trait ColumnBased: StrBased {
  fn col(&self) -> usize;
  fn set_col(self, col: usize) -> Self;
}

/// Class of line-based inputs; i.e. accepting the concepts of a “column” and of a “line.”
pub trait LineBased: ColumnBased {
  fn line(&self) -> usize;
  fn set_line(self, line: usize) -> Self;
}

/// Line-based input around `&str`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineBasedStr<'a> {
  pub input: &'a str,
  pub line: usize,
  pub col: usize,
}

impl<'a> From<&'a str> for LineBasedStr<'a> {
  fn from(input: &'a str) -> Self {
    Self {
      input,
      line: 0,
      col: 0,
    }
  }
}

impl<'a> StrBased for LineBasedStr<'a> {
  fn as_str(&self) -> &str {
    self.input
  }

  fn advance(self, count: usize) -> Self {
    Self {
      input: &self.input[count..],
      ..self
    }
  }
}

impl<'a> ColumnBased for LineBasedStr<'a> {
  fn col(&self) -> usize {
    self.col
  }

  fn set_col(self, col: usize) -> Self {
    Self { col, ..self }
  }
}

impl<'a> LineBased for LineBasedStr<'a> {
  fn line(&self) -> usize {
    self.line
  }

  fn set_line(self, line: usize) -> Self {
    Self { line, ..self }
  }
}
