use crate::text_token::TextToken;
use std::cmp::PartialEq;
use std::str::Bytes;

#[derive(Clone, Debug)]
pub(crate) enum Token {
    Char(CharToken),
    Text(TextToken),
}

impl Token {
    pub(crate) fn match_input(&self, input: &mut Bytes) -> Option<usize> {
        match self {
            Token::Char(token) => token.match_char(&input.next()?).then_some(1),
            Token::Text(token) => token.match_input(input),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum CharToken {
    Exact(u8),
    Wildcard,
    Group(Vec<u8>),
    NegativeGroup(Vec<u8>),
    CharType(CharType),
    StartLine,
    EndLine,
}

impl CharToken {
    pub fn match_char(&self, input_ch: &u8) -> bool {
        match self {
            CharToken::Exact(ch) => input_ch == ch,
            CharToken::CharType(char_type) => char_type.match_char(input_ch),
            CharToken::Group(group) => group.contains(input_ch),
            CharToken::NegativeGroup(group) => !group.contains(input_ch),
            CharToken::Wildcard => true,
            _ => false,
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum CharType {
    Digit,
    Alphanumeric,
    Whitespace,
    NonDigit,
    NonAlphanumeric,
    NonWhitespace,
}

impl CharType {
    pub fn match_char(&self, input_ch: &u8) -> bool {
        match self {
            CharType::Digit => input_ch.is_ascii_digit(),
            CharType::Alphanumeric => input_ch.is_ascii_alphanumeric(),
            CharType::Whitespace => input_ch.is_ascii_whitespace(),
            CharType::NonDigit => !input_ch.is_ascii_digit(),
            CharType::NonAlphanumeric => !input_ch.is_ascii_alphanumeric(),
            CharType::NonWhitespace => !input_ch.is_ascii_whitespace(),
        }
    }
}
