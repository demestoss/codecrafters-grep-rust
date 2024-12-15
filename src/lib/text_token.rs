use std::str::Bytes;

use crate::{pattern::Pattern, pattern_list::PatternList};

#[derive(Clone, Debug)]
pub enum TextToken {
    Alteration(Vec<PatternList>),
}

impl TextToken {
    pub fn match_input(&self, input: &Bytes) -> Option<usize> {
        match self {
            TextToken::Alteration(variants) => {
                let input_length = input.len();
                for variant in variants.clone() {
                    let mut input_clone = input.clone();
                    let mut pattern = Pattern::new(variant);

                    if pattern.match_here(&mut input_clone) {
                        let new_input_length = input_clone.len();
                        return Some(input_length - new_input_length);
                    }
                }
                None
            }
        }
    }
}
