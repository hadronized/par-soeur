use par_soeur::{parse_lexeme, Parser, TopParser};
use std::{collections::HashMap, io::stdin};

#[derive(Clone, Debug, PartialEq)]
enum Value {
  String(String),
  Number(f64),
  Bool(bool),
  Array(Vec<Value>),
  Object(HashMap<String, Value>),
  Null,
}

fn value_parser<'a>() -> TopParser<'a, fn(&'a str) -> Parser<'a, Value, str>, Value, str> {
  TopParser::from_input_parser(value_parser_fn)
}

fn value_parser_fn(input: &str) -> Parser<Value, str> {
  let string_parser = || {
    TopParser::from_input_parser(|input: &str| {
      if input.is_empty() {
        return Parser::NoParse;
      }

      let mut chars = input.chars();
      if chars.next() != Some('"') {
        return Parser::NoParse;
      }

      let mut escaped = false;
      let s: String = chars
        .take_while(|&c| {
          escaped = c == '\'';
          c != '"' && !escaped
        })
        .collect();

      let input = &input[1 + s.len()..];

      if input.is_empty() {
        return Parser::NoParse;
      }

      let input = &input[1..];
      Parser::Parsed { data: s, input }
    })
  };

  let number_parser = TopParser::from_input_parser(|input: &str| {
    let m = input
      .chars()
      .take_while(|&c| c.is_ascii_digit() || "-.".contains(c));
    let m = &input[..m.count()];
    match lexical_core::parse::<f64>(m.as_bytes())
      .ok()
      .map(Value::Number)
    {
      Some(data) => Parser::Parsed {
        data,
        input: &input[m.len()..],
      },
      None => Parser::NoParse,
    }
  });

  let bool_parser = {
    parse_lexeme("true")
      .const_map(true)
      .or(parse_lexeme("false").const_map(false))
      .map(Value::Bool)
  };

  let null_parser = parse_lexeme("null").const_map(Value::Null);

  let ws_parser = || par_soeur::parse_while(char::is_whitespace).opt();

  let array_el_parser = value_parser()
    .delimited0(parse_lexeme(",").left(ws_parser()))
    .left(parse_lexeme(",").left(ws_parser()).opt());
  let array_parser = parse_lexeme("[")
    .left(ws_parser())
    .right(array_el_parser)
    .left(parse_lexeme("]"))
    .map(Value::Array);

  let obj_pair_parser = string_parser()
    .left(ws_parser())
    .left(parse_lexeme(":"))
    .left(ws_parser())
    .zip(value_parser(), |k, v| (k, v));
  let obj_parser = parse_lexeme("{")
    .left(ws_parser())
    .right(obj_pair_parser.delimited0(parse_lexeme(",").left(ws_parser())))
    .left(ws_parser())
    .left(parse_lexeme("}"))
    .map(|kvs| Value::Object(kvs.into_iter().collect()));

  string_parser()
    .map(Value::String)
    .or(number_parser)
    .or(bool_parser)
    .or(null_parser)
    .or(array_parser)
    .or(obj_parser)
    .parse(input)
}

fn main() {
  let mut line = String::new();
  while let Ok(_) = stdin().read_line(&mut line) {
    println!("{:?}", value_parser().parse(&line));
    line.clear();
  }
}
