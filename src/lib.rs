pub mod input;

use std::marker::PhantomData;

pub struct TopParser<'a, F, A, I>
where
  I: 'a,
  F: 'a,
{
  parser: F,
  _phantom: PhantomData<&'a (A, I)>,
}

impl<'a, F, A, I> TopParser<'a, F, A, I>
where
  I: 'a + PartialEq,
  F: 'a + Fn(I) -> Parser<A, I>,
{
  pub fn from_input_parser(f: F) -> Self {
    TopParser {
      parser: f,
      _phantom: PhantomData,
    }
  }

  pub fn parse(&self, input: I) -> Parser<A, I> {
    (self.parser)(input)
  }

  pub fn zip<B, C>(
    self,
    other: TopParser<'a, impl Fn(I) -> Parser<B, I>, B, I>,
    f: impl 'a + Fn(A, B) -> C,
  ) -> TopParser<'a, impl 'a + Fn(I) -> Parser<C, I>, C, I> {
    TopParser {
      parser: move |input| match (self.parser)(input) {
        Parser::Parsed { data, input } => match (other.parser)(input) {
          Parser::Parsed { data: data2, input } => Parser::Parsed {
            data: f(data, data2),
            input,
          },
          Parser::NoParse => Parser::NoParse,
        },
        Parser::NoParse => Parser::NoParse,
      },
      _phantom: PhantomData,
    }
  }

  pub fn left<B>(
    self,
    other: TopParser<'a, impl 'a + Fn(I) -> Parser<B, I>, B, I>,
  ) -> TopParser<'a, impl 'a + Fn(I) -> Parser<A, I>, A, I> {
    TopParser {
      parser: move |input| match (self.parser)(input) {
        Parser::Parsed { data, input } => match (other.parser)(input) {
          Parser::Parsed { input, .. } => Parser::Parsed { data, input },
          Parser::NoParse => Parser::NoParse,
        },
        Parser::NoParse => Parser::NoParse,
      },
      _phantom: PhantomData,
    }
  }

  pub fn right<B>(
    self,
    other: TopParser<'a, impl 'a + Fn(I) -> Parser<B, I>, B, I>,
  ) -> TopParser<'a, impl 'a + Fn(I) -> Parser<B, I>, B, I> {
    TopParser {
      parser: move |input| match (self.parser)(input) {
        Parser::Parsed { input, .. } => match (other.parser)(input) {
          Parser::Parsed { data, input } => Parser::Parsed { data, input },
          Parser::NoParse => Parser::NoParse,
        },
        Parser::NoParse => Parser::NoParse,
      },
      _phantom: PhantomData,
    }
  }

  pub fn and_then<B, G>(
    self,
    f: impl Fn(A) -> TopParser<'a, G, B, I>,
  ) -> TopParser<'a, impl 'a + Fn(I) -> Parser<B, I>, B, I>
  where
    G: 'a + Fn(I) -> Parser<B, I>,
  {
    TopParser {
      parser: move |input| match (self.parser)(input) {
        Parser::Parsed { data, input } => (f(data).parser)(input),
        Parser::NoParse => Parser::NoParse,
      },
      _phantom: PhantomData,
    }
  }

  pub fn map<B>(
    self,
    f: impl 'a + Fn(A) -> B,
  ) -> TopParser<'a, impl 'a + Fn(I) -> Parser<B, I>, B, I> {
    TopParser {
      parser: move |input| match (self.parser)(input) {
        Parser::Parsed { data, input } => Parser::Parsed {
          data: f(data),
          input,
        },
        Parser::NoParse => Parser::NoParse,
      },
      _phantom: PhantomData,
    }
  }

  pub fn const_map<B>(self, b: B) -> TopParser<'a, impl 'a + Fn(I) -> Parser<B, I>, B, I>
  where
    B: Clone,
  {
    TopParser {
      parser: move |input| match (self.parser)(input) {
        Parser::Parsed { input, .. } => Parser::Parsed {
          data: b.clone(),
          input,
        },
        Parser::NoParse => Parser::NoParse,
      },
      _phantom: PhantomData,
    }
  }

  pub fn many0(self) -> TopParser<'a, impl 'a + Fn(I) -> Parser<Vec<A>, I>, Vec<A>, I>
  where
    I: Clone,
  {
    TopParser {
      parser: move |mut i: I| {
        let mut results = Vec::new();

        while let Parser::Parsed { data, input } = (self.parser)(i.clone()) {
          if input == i {
            // input hasn’t changed, which might indicate that the parser didn’t consume; break
            break;
          }

          results.push(data);
          i = input;
        }

        Parser::Parsed {
          input: i,
          data: results,
        }
      },
      _phantom: PhantomData,
    }
  }

  pub fn many1(self) -> TopParser<'a, impl 'a + Fn(I) -> Parser<Vec<A>, I>, Vec<A>, I>
  where
    I: Clone,
  {
    TopParser {
      parser: move |mut i: I| {
        let mut results = Vec::new();

        while let Parser::Parsed { data, input } = (self.parser)(i.clone()) {
          // input hasn’t changed, which might indicate that the parser didn’t consume; break
          if input == i {
            break;
          }

          results.push(data);
          i = input;
        }

        if results.is_empty() {
          Parser::NoParse
        } else {
          Parser::Parsed {
            input: i,
            data: results,
          }
        }
      },
      _phantom: PhantomData,
    }
  }

  pub fn opt(self) -> TopParser<'a, impl 'a + Fn(I) -> Parser<Option<A>, I>, Option<A>, I>
  where
    I: Clone,
  {
    TopParser {
      parser: move |input: I| match (self.parser)(input.clone()) {
        Parser::Parsed { data, input } => Parser::Parsed {
          data: Some(data),
          input,
        },
        Parser::NoParse => Parser::Parsed { data: None, input },
      },
      _phantom: PhantomData,
    }
  }

  pub fn or(
    self,
    other: TopParser<'a, impl 'a + Fn(I) -> Parser<A, I>, A, I>,
  ) -> TopParser<'a, impl 'a + Fn(I) -> Parser<A, I>, A, I>
  where
    I: Clone,
  {
    TopParser {
      parser: move |input: I| match (self.parser)(input.clone()) {
        Parser::NoParse => (other.parser)(input),
        p => p,
      },
      _phantom: PhantomData,
    }
  }

  pub fn delimited0<B>(
    self,
    delimiter: TopParser<'a, impl 'a + Fn(I) -> Parser<B, I>, B, I>,
  ) -> TopParser<'a, impl 'a + Fn(I) -> Parser<Vec<A>, I>, Vec<A>, I>
  where
    I: Clone,
  {
    TopParser::from_input_parser(move |mut i: I| {
      let mut even = true;
      let mut results = Vec::new();

      loop {
        if even {
          if let Parser::Parsed { data, input } = self.parse(i.clone()) {
            results.push(data);
            i = input;
          } else {
            break;
          }
        } else {
          if let Parser::Parsed { input, .. } = delimiter.parse(i.clone()) {
            i = input;
          } else {
            break;
          }
        }

        even = !even;
      }

      if !even || results.is_empty() {
        Parser::Parsed {
          data: results,
          input: i,
        }
      } else {
        Parser::NoParse
      }
    })
  }

  pub fn delimited1<B>(
    self,
    delimiter: TopParser<'a, impl 'a + Fn(I) -> Parser<B, I>, B, I>,
  ) -> TopParser<'a, impl 'a + Fn(I) -> Parser<Vec<A>, I>, Vec<A>, I>
  where
    I: Clone,
  {
    TopParser::from_input_parser(move |mut i: I| {
      let mut even = true;
      let mut results = Vec::new();

      loop {
        if even {
          if let Parser::Parsed { data, input } = self.parse(i.clone()) {
            results.push(data);
            i = input;
          } else {
            break;
          }
        } else {
          if let Parser::Parsed { input, .. } = delimiter.parse(i.clone()) {
            i = input;
          } else {
            break;
          }
        }

        even = !even;
      }

      if even {
        Parser::NoParse
      } else {
        Parser::Parsed {
          data: results,
          input: i,
        }
      }
    })
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Parser<A, I> {
  Parsed { data: A, input: I },
  NoParse,
}

impl<'a, A, I> Parser<A, I> {
  pub fn ok(self) -> Option<A> {
    match self {
      Parser::Parsed { data, .. } => Some(data),
      Parser::NoParse => None,
    }
  }
}

impl<'a, A, I> From<Option<Parser<A, I>>> for Parser<A, I> {
  fn from(x: Option<Parser<A, I>>) -> Self {
    x.unwrap_or_else(|| Parser::NoParse)
  }
}

pub fn parse_u32<'a>() -> TopParser<'a, impl Fn(&'a str) -> Parser<u32, &'a str>, u32, &'a str> {
  TopParser::from_input_parser(|input: &'a str| {
    let mut count = 0;
    let len = input.len();
    let bytes = input.as_bytes();

    while count < len && bytes[count].is_ascii_digit() {
      count += 1;
    }

    input[..count]
      .parse()
      .ok()
      .map(|data| Parser::Parsed {
        data,
        input: &input[count..],
      })
      .into()
  })
}

pub fn parse_spaces<'a>() -> TopParser<'a, impl Fn(&'a str) -> Parser<(), &'a str>, (), &'a str> {
  TopParser::from_input_parser(|input: &'a str| {
    let mut count = 0;
    let len = input.len();
    let bytes = input.as_bytes();

    while count < len && bytes[count].is_ascii_whitespace() {
      count += 1;
    }

    Parser::Parsed {
      data: (),
      input: &input[count..],
    }
  })
}

pub fn parse_lexeme<'a>(
  l: &'a str,
) -> TopParser<'a, impl Fn(&'a str) -> Parser<(), &'a str>, (), &'a str> {
  TopParser {
    parser: move |input: &'a str| {
      if input.starts_with(l) {
        Parser::Parsed {
          data: (),
          input: &input[l.len()..],
        }
      } else {
        Parser::NoParse
      }
    },
    _phantom: PhantomData,
  }
}

pub fn parse_take<'a>(
  count: usize,
) -> TopParser<'a, impl Fn(&'a str) -> Parser<&'a str, &'a str>, &'a str, &'a str> {
  TopParser {
    parser: move |input: &'a str| {
      if input.len() >= count {
        Parser::Parsed {
          data: &input[..count],
          input: &input[count..],
        }
      } else {
        Parser::NoParse
      }
    },
    _phantom: PhantomData,
  }
}

pub fn parse_while<'a>(
  predicate: impl Fn(char) -> bool,
) -> TopParser<'a, impl Fn(&'a str) -> Parser<&'a str, &'a str>, &'a str, &'a str> {
  TopParser {
    parser: move |input: &'a str| {
      let count = input.chars().take_while(|c| predicate(*c)).count();

      if count == 0 {
        Parser::NoParse
      } else {
        Parser::Parsed {
          data: &input[..count],
          input: &input[count..],
        }
      }
    },
    _phantom: PhantomData,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_u32_test() {
    assert_eq!(
      parse_u32().parse("123lol"),
      Parser::Parsed {
        data: 123,
        input: "lol",
      }
    );
  }

  #[test]
  fn parse_spaces_test() {
    assert_eq!(
      parse_spaces().parse("       lol"),
      Parser::Parsed {
        data: (),
        input: "lol",
      }
    );
  }

  #[test]
  fn and_then_test() {
    let parser = parse_u32()
      .and_then(|data| parse_spaces().const_map(data))
      .and_then(|data| parse_u32().map(move |data2| (data, data2)));

    assert_eq!(
      parser.parse("123   456lol"),
      Parser::Parsed {
        data: (123, 456),
        input: "lol",
      }
    );
  }

  #[test]
  fn many0_test() {
    let parser = parse_u32()
      .and_then(|data| parse_spaces().const_map(data))
      .many0();

    assert_eq!(
      parser.parse("1 2 3 4 lol"),
      Parser::Parsed {
        data: vec![1, 2, 3, 4],
        input: "lol",
      }
    );

    assert_eq!(
      parser.parse("lol"),
      Parser::Parsed {
        data: Vec::new(),
        input: "lol"
      }
    );
  }

  #[test]
  fn many1_test() {
    let parser = parse_u32()
      .and_then(|data| parse_spaces().const_map(data))
      .many1();

    assert_eq!(
      parser.parse("1 2 3 4 lol"),
      Parser::Parsed {
        data: vec![1, 2, 3, 4],
        input: "lol",
      }
    );

    assert_eq!(parser.parse("lol"), Parser::NoParse);
  }

  #[test]
  fn opt_test() {
    let parser = parse_spaces().and_then(|_| parse_u32().opt()).many1();

    assert_eq!(
      parser.parse("    1  2 3   "),
      Parser::Parsed {
        data: vec![Some(1), Some(2), Some(3), None],
        input: "",
      }
    );
  }

  #[test]
  fn lexeme_test() {
    let parser = parse_lexeme("foo");

    assert_eq!(
      parser.parse("foobarzoo"),
      Parser::Parsed {
        data: (),
        input: "barzoo",
      }
    );
  }

  #[test]
  fn or_test() {
    let parser = parse_lexeme("foo").or(parse_lexeme("bar"));

    assert_eq!(
      parser.parse("foo"),
      Parser::Parsed {
        data: (),
        input: "",
      }
    );

    assert_eq!(
      parser.parse("bar"),
      Parser::Parsed {
        data: (),
        input: "",
      }
    );

    let parser = parser.many1();

    assert_eq!(
      parser.parse("foobar"),
      Parser::Parsed {
        data: vec![(), ()],
        input: "",
      }
    );
  }

  #[test]
  fn zip_test() {
    let parser = parse_u32().zip(parse_take(3), |n, l| (n, l));

    assert_eq!(
      parser.parse("123foolol"),
      Parser::Parsed {
        data: (123, "foo"),
        input: "lol",
      }
    );
  }

  #[test]
  fn left_test() {
    let parser = parse_u32().left(parse_spaces());

    assert_eq!(
      parser.parse("123  lol"),
      Parser::Parsed {
        data: 123,
        input: "lol",
      }
    );
  }

  #[test]
  fn right_test() {
    let parser = parse_spaces().right(parse_u32());

    assert_eq!(
      parser.parse("   123lol"),
      Parser::Parsed {
        data: 123,
        input: "lol",
      }
    );
  }

  #[test]
  fn applicative_person_test() {
    #[derive(Debug, Eq, PartialEq)]
    struct Person {
      name: String,
      age: u32,
    }

    let parser =
      parse_while(char::is_alphabetic)
        .left(parse_spaces())
        .zip(parse_u32(), |name, age| Person {
          name: name.to_owned(),
          age,
        });

    let expected = Person {
      name: "Henry".to_owned(),
      age: 48,
    };
    assert_eq!(
      parser.parse("Henry 48lol"),
      Parser::Parsed {
        data: expected,
        input: "lol",
      }
    );
  }
}
