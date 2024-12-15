use std::str::{Bytes, FromStr};

use crate::{pattern_item::PatternItem, pattern_list::PatternList, token::CharToken};

pub struct Pattern {
    pattern: PatternList,
}

impl Pattern {
    pub fn new(pattern_list: PatternList) -> Self {
        Self {
            pattern: pattern_list,
        }
    }

    pub fn match_line(&mut self, input: &str) -> anyhow::Result<bool> {
        let res = self.match_line_proceed(input);
        self.pattern.reset();
        res
    }

    fn match_line_proceed(&mut self, input: &str) -> anyhow::Result<bool> {
        let mut input = input.bytes();

        if self.pattern.is_next_token(CharToken::StartLine) {
            self.pattern.next();
            return Ok(self.match_here(&mut input));
        }

        loop {
            // Clone because we can start pattern from the start if it failed at some point 
            // without changing the input
            if self.match_here(&mut input.clone()) {
                return Ok(true);
            }
            if input.next().is_none() {
                return Ok(false);
            }
            self.pattern.reset();
        }
    }

    pub fn match_here(&mut self, input: &mut Bytes) -> bool {
        if self.pattern.is_next_optional() && input.len() == 0 {
            return true;
        }
        dbg!(&input, self.pattern.clone());
        if self.pattern.is_next_token(CharToken::EndLine) && input.len() == 0 {
            return true;
        }
        let Some(pattern_item) = self.pattern.next() else {
            return true;
        };
        let pattern_item = pattern_item.clone();

        let Some(skip_count) = self.handle_match_option(pattern_item, &mut input.clone()) else {
            return false;
        };

        for _ in 0..skip_count {
            input.next();
        }
        self.match_here(input)
    }

    fn handle_match_option(&self, pattern_item: PatternItem, input: &mut Bytes) -> Option<usize> {
        let match_option = pattern_item.match_input(input);
        let mut skip_count = match_option?;

        if skip_count == 0 {
            return Some(0);
        }

        if pattern_item.is_multiple_match() {
            let res = self.match_more(input, pattern_item);
            match res {
                None => return None,
                Some(count) => skip_count += count,
            }
        }

        Some(skip_count)
    }

    fn match_more(&self, input: &mut Bytes, pattern_item: PatternItem) -> Option<usize> {
        let mut skip_count = 0;
        let mut match_times = 1;
        while pattern_item.can_match_more(match_times) {
            if input.len() == 0 {
                break;
            };
            let Some(match_count) = pattern_item.match_input(input) else {
                break;
            };
            skip_count += match_count;
            match_times += 1;

            if pattern_item.is_least_matched(match_times) {
                if let Some(next_pattern_item) = self.pattern.peek() {
                    if next_pattern_item.match_input(&mut input.clone()).is_some() {
                        break;
                    }
                }
            }
        }

        if pattern_item.is_least_matched(match_times) {
            Some(skip_count)
        } else {
            None
        }
    }
}

impl FromStr for Pattern {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            pattern: PatternList::from_str(s)?,
        })
    }
}
