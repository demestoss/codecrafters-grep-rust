use anyhow::{anyhow, bail, Context};
use std::cmp::PartialEq;
use std::iter::{Enumerate, Peekable};
use std::ops::Index;
use std::str::{Bytes, FromStr};

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
            CharToken::CharType(char_type) => char_type.match_char(&input_ch),
            CharToken::Group(group) => group.contains(input_ch),
            CharToken::NegativeGroup(group) => !group.contains(input_ch),
            CharToken::Wildcard => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum TextToken {
    Whitespace,
    Alteration(Vec<Vec<u8>>),
}

impl TextToken {
    pub fn match_input(&self, input: &mut Bytes) -> Option<usize> {
        match self {
            TextToken::Whitespace => match input.take_while(|c| c.is_ascii_whitespace()).count() {
                0 => None,
                c => Some(c),
            },
            TextToken::Alteration(variants) => {
                for variant in variants {
                    let mut input_clone = input.clone();
                    if variant.iter().all(|&v| input_clone.next() == Some(v)) {
                        return Some(variant.len());
                    }
                }
                None
            }
        }
    }
}

enum TokenModifier {
    Optional,
    OneOrMore,
    Exact(usize),
    AtLeast(usize),
    Between(usize, usize),
}

#[derive(Clone, Debug)]
enum Token {
    Char(CharToken),
    Text(TextToken),
}

impl Token {
    fn match_input(&self, input: &mut Bytes) -> Option<usize> {
        match self {
            Token::Char(token) => token.match_char(&input.next()?).then_some(1),
            Token::Text(token) => token.match_input(input),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PatternItem {
    token: Token,
    optional: bool,
    more_than: Option<usize>,
    less_than: Option<usize>,
}

impl PatternItem {
    fn new(token: CharToken) -> Self {
        Self {
            token: Token::Char(token),
            optional: false,
            more_than: None,
            less_than: None,
        }
    }

    fn new_text(token: TextToken) -> Self {
        Self {
            token: Token::Text(token),
            optional: false,
            more_than: None,
            less_than: None,
        }
    }

    fn apply_modifier(&mut self, modifier: TokenModifier) {
        match modifier {
            TokenModifier::Optional => self.optional = true,
            TokenModifier::OneOrMore => self.more_than = Some(1),
            TokenModifier::Exact(exact) => {
                self.more_than = Some(exact);
                self.less_than = Some(exact);
            }
            TokenModifier::AtLeast(at_least) => self.more_than = Some(at_least),
            TokenModifier::Between(more_than, less_than) => {
                self.more_than = Some(more_than);
                self.less_than = Some(less_than);
            }
        }
    }

    pub fn match_input(&self, input: &mut Bytes) -> Option<usize> {
        match self.token.match_input(input) {
            Some(count) => Some(count),
            None => {
                if self.optional {
                    Some(0)
                } else {
                    None
                }
            }
        }
    }

    pub fn is_multiple_match(&self) -> bool {
        self.more_than.is_some() || self.less_than.is_some()
    }

    pub fn is_least_matched(&self, current_times: usize) -> bool {
        let more_than = self.more_than.unwrap_or(1);
        current_times >= more_than
    }

    pub fn can_match_more(&self, current_times: usize) -> bool {
        if let Some(less_than) = self.less_than {
            current_times + 1 <= less_than
        } else {
            true
        }
    }
}

#[derive(Clone, Debug)]
pub struct Pattern {
    inner: Vec<PatternItem>,
    cursor: usize,
}

impl Pattern {
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
}

impl FromStr for Pattern {
    type Err = anyhow::Error;

    fn from_str(pattern: &str) -> Result<Self, Self::Err> {
        let mut inner: Vec<PatternItem> = Vec::new();

        let length = pattern.len();
        let mut pattern = pattern.bytes().into_iter().enumerate().peekable();

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
                inner.push(PatternItem::new(CharToken::StartLine))
            } else if char == b'$' && i == length - 1 {
                inner.push(PatternItem::new(CharToken::EndLine))
            } else if char == b'.' {
                inner.push(PatternItem::new(CharToken::Wildcard))
            } else if char == b'*' {
                let mut item = PatternItem::new(CharToken::Wildcard);
                item.apply_modifier(TokenModifier::OneOrMore);
                inner.push(item);
                match pattern.peek() {
                    Some((_, b'.')) => {
                        inner.push(PatternItem::new(CharToken::Exact(b'.')));
                        pattern.next();
                    }
                    _ => (),
                }
            } else if char == b'\\' {
                let (_, next_char) = pattern.next().ok_or(anyhow!(
                    "incorrect pattern: \\ symbol without value after it"
                ))?;
                match next_char {
                    b'd' => inner.push(PatternItem::new(CharToken::CharType(CharType::Digit))),
                    b'w' => inner.push(PatternItem::new(CharToken::CharType(
                        CharType::Alphanumeric,
                    ))),
                    b's' => inner.push(PatternItem::new_text(TextToken::Whitespace)),
                    _ => inner.push(PatternItem::new(CharToken::Exact(next_char))),
                }
            } else if char == b'(' {
                let group = parse_group(&mut pattern, b')')?;

                let alterations = group
                    .split(|&c| c == b'|')
                    .map(Vec::from)
                    .collect::<Vec<_>>();
                inner.push(PatternItem::new_text(TextToken::Alteration(alterations)));
            } else if char == b'[' {
                let mut group = parse_group(&mut pattern, b']')?;

                if group[0] == b'^' {
                    group.remove(0);
                    inner.push(PatternItem::new(CharToken::NegativeGroup(group)))
                } else {
                    inner.push(PatternItem::new(CharToken::Group(group)))
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
                inner.push(PatternItem::new(CharToken::Exact(char)))
            }
        }

        Ok(Self { inner, cursor: 0 })
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

impl Index<usize> for Pattern {
    type Output = PatternItem;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}
