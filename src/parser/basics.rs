// Copyright (c) 2020 White Leaf
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use nom::bytes::complete::{tag, take_till1, take_while, take_while1};
use nom::character::complete::{char, digit1};
use nom::combinator::map_res;
use nom::{sequence::delimited, IResult};

pub(crate) fn parse_ident(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-')(input)
}

pub(crate) fn parse_string(input: &str) -> IResult<&str, &str> {
    delimited(char('\''), take_till1(|c: char| c == '\''), char('\''))(input)
}

pub(crate) fn parse_number(input: &str) -> IResult<&str, i64> {
    map_res(digit1, |s: &str| s.parse::<i64>())(input)
}

pub(crate) fn parse_separator(input: &str) -> IResult<&str, &str> {
    delimited(
        take_while(|c: char| c == ' '),
        tag(","),
        take_while(|c: char| c == ' '),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_idents() {
        let parsed = parse_ident("ident_is-ok");
        let expected = ("", "ident_is-ok");

        assert_eq!(parsed, Ok(expected));

        let parsed = parse_ident("this is not ok");
        let expected = (" is not ok", "this");

        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn test_parse_string() {
        let parsed = parse_string("'holo, cómo estás?'");
        let expected = ("", "holo, cómo estás?");

        assert_eq!(parsed, Ok(expected));

        let parsed = parse_string("'holo' #wed2@ws");
        let expected = (" #wed2@ws", "holo");

        assert_eq!(parsed, Ok(expected))
    }

    #[test]
    fn test_parse_numbers() {
        let parsed = parse_number("12345");
        let expected = ("", 12345);

        assert_eq!(parsed, Ok(expected));

        let parsed = parse_number("12c3");
        let expected = ("c3", 12);
        assert_eq!(parsed, Ok(expected));
    }
}
