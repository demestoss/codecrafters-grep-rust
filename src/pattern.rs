use anyhow::{anyhow, bail};
use std::cmp::PartialEq;
use std::ops::Index;

#[derive(PartialEq, Clone, Debug)]
pub enum CharType {
    Digit,
    Alphanumeric,
}

impl CharType {
    pub fn match_char(&self, input_ch: &u8) -> bool {
        match self {
            CharType::Digit => input_ch.is_ascii_digit(),
            CharType::Alphanumeric => input_ch.is_ascii_alphanumeric(),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Token {
    CharExact(u8),
    Group(Vec<u8>),
    NegativeGroup(Vec<u8>),
    CharType(CharType),
    StartLine,
    EndLine,
}

impl Token {
    pub fn match_char(&self, input_ch: &u8) -> bool {
        match self {
            Token::CharExact(ch) => input_ch == ch,
            Token::CharType(char_type) => char_type.match_char(&input_ch),
            Token::Group(group) => group.contains(input_ch),
            Token::NegativeGroup(group) => !group.contains(input_ch),
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PatternItem {
    token: Token,
    optional: bool,
    multiple: bool,
}

impl PatternItem {
    fn new(token: Token) -> Self {
        Self {
            token,
            optional: false,
            multiple: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Pattern {
    inner: Vec<PatternItem>,
    cursor: usize,
}

impl Pattern {
    pub fn parse(pattern: &str) -> anyhow::Result<Self> {
        let mut inner = Vec::new();

        let length = pattern.len();
        let mut pattern = pattern.bytes().into_iter().enumerate();

        while let Some((i, char)) = pattern.next() {
            if char == b'?' {
                inner.push(PatternItem::Modifier(Modifier::Optional))
            } else if char == b'+' {
                inner.push(PatternItem::Modifier(Modifier::OneOrMore))
            } else if char == b'^' && i == 0 {
                inner.push(PatternItem::Token(Token::StartLine))
            } else if char == b'$' && i == length - 1 {
                inner.push(PatternItem::Token(Token::EndLine))
            } else if char == b'\\' {
                let (_, next_char) = pattern.next().ok_or(anyhow!(
                    "incorrect pattern: \\ symbol without value after it"
                ))?;
                match next_char {
                    b'd' => inner.push(PatternItem::Token(Token::CharType(CharType::Digit))),
                    b'w' => inner.push(PatternItem::Token(Token::CharType(CharType::Alphanumeric))),
                    _ => inner.push(PatternItem::Token(Token::CharExact(next_char))),
                }
            } else if char == b'[' {
                let mut group = Vec::new();
                let (_, mut char) = pattern
                    .next()
                    .ok_or(anyhow!("incorrect pattern group: group open without end"))?;
                while char != b']' {
                    group.push(char);
                    char = pattern.next().ok_or(anyhow!("incorrect group pattern"))?.1;
                }
                if group.is_empty() {
                    bail!("incorrect group pattern: empty group")
                }

                if group[0] == b'^' {
                    group.pop();
                    inner.push(PatternItem::Token(Token::NegativeGroup(group)))
                } else {
                    inner.push(PatternItem::Token(Token::Group(group)))
                }
            } else {
                inner.push(PatternItem::Token(Token::CharExact(char)))
            }
        }

        Ok(Self { inner, cursor: 0 })
    }

    pub fn is_next_token(&self, token_to_check: Token) -> bool {
        let t = self.inner.get(self.cursor);
        match t {
            Some(PatternItem::Token(token)) => *token == token_to_check,
            _ => false,
        }
    }

    pub fn is_next_modifier(&self, modifier: Modifier) -> bool {
        let t = self.inner.get(self.cursor);
        match t {
            Some(PatternItem::Modifier(m)) => *m == modifier,
            _ => false,
        }
    }

    pub fn is_next_optional(&self) -> bool {
        let token = self.inner.get(self.cursor);
        let Some(PatternItem::Modifier(modifier)) = self.inner.get(self.cursor + 1) else {
            return false;
        };
        token.is_some() && *modifier == Modifier::Optional
    }

    pub fn next(&mut self) -> Option<&PatternItem> {
        let val = self.inner.get(self.cursor);
        self.cursor += 1;
        val
    }
}

impl Index<usize> for Pattern {
    type Output = PatternItem;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}
