use crate::pattern_item::PatternItem;
use crate::text_token::TextToken;
use crate::token::{CharToken, CharType, Token};
use crate::token_modifier::TokenModifier;
use anyhow::Context;
use anyhow::{anyhow, bail};
use std::ops::Index;
use std::{
    iter::{Enumerate, Peekable},
    str::{Bytes, FromStr},
};

#[derive(Clone, Debug)]
pub struct PatternList {
    inner: Vec<PatternItem>,
    cursor: usize,
}

impl PatternList {
    pub fn is_next_token(&self, token_to_check: CharToken) -> bool {
        let t = self.inner.get(self.cursor);
        match t {
            Some(PatternItem {
                token: Token::Char(token),
                ..
            }) => *token == token_to_check,
            _ => false,
        }
    }

    pub fn is_next_optional(&self) -> bool {
        let Some(pattern_item) = self.inner.get(self.cursor) else {
            return false;
        };
        pattern_item.optional
    }

    pub fn next(&mut self) -> Option<&PatternItem> {
        let val = self.inner.get(self.cursor);
        self.cursor += 1;
        val
    }

    pub fn peek(&self) -> Option<&PatternItem> {
        self.inner.get(self.cursor)
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
    }
}

impl FromStr for PatternList {
    type Err = anyhow::Error;

    fn from_str(pattern: &str) -> Result<Self, Self::Err> {
        let mut inner: Vec<PatternItem> = Vec::new();

        let length = pattern.len();
        let mut pattern = pattern.bytes().enumerate().peekable();

        while let Some((i, char)) = pattern.next() {
            if char == b'?' {
                let Some(item) = inner.last_mut() else {
                    bail!("incorrect modifier usage: '?' used without token")
                };
                item.apply_modifier(TokenModifier::Optional);
            } else if char == b'+' {
                let Some(item) = inner.last_mut() else {
                    bail!("incorrect modifier usage: '+' used without token")
                };
                item.apply_modifier(TokenModifier::OneOrMore);
            } else if char == b'^' && i == 0 {
                inner.push(PatternItem::new_char(CharToken::StartLine))
            } else if char == b'$' && i == length - 1 {
                inner.push(PatternItem::new_char(CharToken::EndLine))
            } else if char == b'.' {
                inner.push(PatternItem::new_char(CharToken::Wildcard))
            } else if char == b'*' {
                let mut item = PatternItem::new_char(CharToken::Wildcard);
                item.apply_modifier(TokenModifier::OneOrMore);
                inner.push(item);
                if let Some((_, b'.')) = pattern.peek() {
                    inner.push(PatternItem::new_char(CharToken::Exact(b'.')));
                    pattern.next();
                }
            } else if char == b'\\' {
                let var_name = anyhow!("incorrect pattern: \\ symbol without value after it");
                let (_, next_char) = pattern.next().ok_or(var_name)?;
                match next_char {
                    b'd' => inner.push(PatternItem::new_char_type(CharType::Digit)),
                    b'D' => inner.push(PatternItem::new_char_type(CharType::NonDigit)),
                    b'w' => inner.push(PatternItem::new_char_type(CharType::Alphanumeric)),
                    b'W' => inner.push(PatternItem::new_char_type(CharType::NonAlphanumeric)),
                    b's' => inner.push(PatternItem::new_char_type(CharType::Whitespace)),
                    b'S' => inner.push(PatternItem::new_char_type(CharType::NonWhitespace)),
                    _ => inner.push(PatternItem::new_char(CharToken::Exact(next_char))),
                }
            } else if char == b'(' {
                let group = parse_group(&mut pattern, b')')?;
                let alterations = group
                    .split(|&c| c == b'|')
                    .flat_map(|v| {
                        let str = String::from_utf8(v.to_vec())?;
                        PatternList::from_str(&str)
                    })
                    .collect::<Vec<_>>();
                inner.push(PatternItem::new_text(TextToken::Alteration(alterations)));
            } else if char == b'[' {
                let mut group = parse_group(&mut pattern, b']')?;

                if group[0] == b'^' {
                    group.remove(0);
                    inner.push(PatternItem::new_char(CharToken::NegativeGroup(group)))
                } else {
                    inner.push(PatternItem::new_char(CharToken::Group(group)))
                }
            } else if char == b'{' {
                let group = parse_group(&mut pattern, b'}')?;
                let group = String::from_utf8(group).context("parse quantifiers group to UTF-8")?;
                let modifier = if let Some((more_than, less_than)) = group.split_once(',') {
                    let at_least = more_than
                        .parse::<usize>()
                        .context("quantifiers 'at least' value parse")?;
                    if less_than.is_empty() {
                        TokenModifier::AtLeast(at_least)
                    } else {
                        let less_than = less_than
                            .parse::<usize>()
                            .context("quantifiers 'at least' value parse")?;
                        TokenModifier::Between(at_least, less_than)
                    }
                } else {
                    let exact = group
                        .parse::<usize>()
                        .context("quantifiers exact value parse")?;
                    TokenModifier::Exact(exact)
                };

                if let Some(item) = inner.last_mut() {
                    item.apply_modifier(modifier);
                } else {
                    bail!("incorrect quantifiers usage: used without token");
                }
            } else {
                inner.push(PatternItem::new_char(CharToken::Exact(char)))
            }
        }

        Ok(Self { inner, cursor: 0 })
    }
}

impl Index<usize> for PatternList {
    type Output = PatternItem;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

fn parse_group(pattern: &mut Peekable<Enumerate<Bytes>>, end_char: u8) -> anyhow::Result<Vec<u8>> {
    let mut group = Vec::new();
    let (_, mut char) = pattern.next().ok_or(anyhow!(
        "incorrect pattern group: group open without end {end_char}"
    ))?;
    while char != end_char {
        group.push(char);
        char = pattern.next().ok_or(anyhow!("incorrect group pattern"))?.1;
    }
    if group.is_empty() {
        bail!("incorrect group pattern: empty group {end_char}")
    }
    Ok(group)
}
