use crate::pattern::{CharToken, Pattern, PatternItem};
use std::str::{Bytes, FromStr};

pub fn match_pattern(input: &str, pattern: &str) -> anyhow::Result<bool> {
    let mut input = input.bytes();
    let mut pattern = Pattern::from_str(pattern)?;

    if pattern.is_next_token(CharToken::StartLine) {
        pattern.next();
        return Ok(match_here(&mut input, &mut pattern));
    }

    loop {
        if match_here(&mut input.clone(), &mut pattern.clone()) {
            return Ok(true);
        }
        if input.next().is_none() {
            return Ok(false);
        }
    }
}

pub fn match_here(input: &mut Bytes, pattern: &mut Pattern) -> bool {
    if pattern.is_next_optional() && input.len() == 0 {
        return true;
    }
    if pattern.is_next_token(CharToken::EndLine) && input.len() == 0 {
        return true;
    }
    let Some(pattern_item) = pattern.next() else {
        return true;
    };

    let Some(skip_count) = handle_match_option(&pattern_item.clone(), &mut input.clone(), pattern)
    else {
        return false;
    };

    for _ in 0..skip_count {
        input.next();
    }
    return match_here(input, pattern);
}

fn handle_match_option(
    pattern_item: &PatternItem,
    input: &mut Bytes,
    pattern: &Pattern,
) -> Option<usize> {
    let match_option = pattern_item.match_input(input);
    let Some(mut skip_count) = match_option else {
        return None;
    };
    if skip_count == 0 {
        return Some(0);
    }

    if pattern_item.is_multiple_match() {
        let res = match_more(input, &pattern_item.clone(), pattern);
        match res {
            None => return None,
            Some(count) => skip_count += count,
        }
    }

    Some(skip_count)
}

fn match_more(input: &mut Bytes, pattern_item: &PatternItem, pattern: &Pattern) -> Option<usize> {
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
            if let Some(next_pattern_item) = pattern.peek() {
                if let Some(_) = next_pattern_item.match_input(&mut input.clone()) {
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
