use std::str::Bytes;

use crate::text_token::TextToken;
use crate::token::{CharToken, CharType, Token};
use crate::token_modifier::TokenModifier;

#[derive(Clone, Debug)]
pub struct PatternItem {
    pub(crate) token: Token,
    pub optional: bool,
    more_than: Option<usize>,
    less_than: Option<usize>,
}

impl PatternItem {
    pub(crate) fn new(token: Token) -> Self {
        Self {
            token,
            optional: false,
            more_than: None,
            less_than: None,
        }
    }

    pub fn new_text(token: TextToken) -> Self {
        Self::new(Token::Text(token))
    }

    pub fn new_char(char: CharToken) -> Self {
        Self::new(Token::Char(char))
    }

    pub fn new_char_type(char: CharType) -> Self {
        Self::new(Token::Char(CharToken::CharType(char)))
    }

    pub fn apply_modifier(&mut self, modifier: TokenModifier) {
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
