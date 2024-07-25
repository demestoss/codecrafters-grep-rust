mod pattern;

use crate::pattern::{Modifier, Pattern, PatternItem, Token};
use anyhow::bail;
use std::env;
use std::io;
use std::process;
use std::str::Bytes;

fn match_pattern(input: &str, pattern: &str) -> anyhow::Result<bool> {
    let mut input = input.bytes();
    let mut pattern = Pattern::parse(pattern)?;

    if pattern.is_next_token(Token::StartLine) {
        pattern.next();
        return match_here(&mut input, &mut pattern);
    }

    loop {
        if match_here(&mut input.clone(), &mut pattern.clone())? {
            return Ok(true);
        }
        if input.next().is_none() {
            return Ok(false);
        }
    }
}

fn match_here(input: &mut Bytes, pattern: &mut Pattern) -> anyhow::Result<bool> {
    if pattern.is_next_optional() && input.len() == 0 {
        return Ok(true);
    }
    if pattern.is_next_token(Token::EndLine) && input.len() == 0 {
        return Ok(true);
    }
    let Some(pattern_item) = pattern.next() else {
        return Ok(true);
    };

    let Some(input_ch) = input.next() else {
        return Ok(false);
    };

    let PatternItem::Token(token) = pattern_item else {
        bail!("unexpected modifier found")
    };
    let token = token.clone();
    let mut is_token_matches = token.match_char(&input_ch);

    if pattern.is_next_modifier(Modifier::OneOrMore) {
        pattern.next();
        if is_token_matches {
            match_one_or_more(input, &token);
        }
    }
    if pattern.is_next_modifier(Modifier::Optional) {
        pattern.next();
    }

    if is_token_matches {
        match_here(input, pattern)
    } else {
        Ok(false)
    }
}

fn match_one_or_more(input: &mut Bytes, token: &Token) {
    let skip_chars_count = input
        .clone()
        .take_while(|&input_ch| token.match_char(&input_ch))
        .count();
    for _ in 0..skip_chars_count {
        input.next();
    }
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    let input_line = input_line.trim_end();
    let res = match_pattern(&input_line, &pattern);

    match res {
        Ok(res) => {
            if res {
                process::exit(0)
            } else {
                process::exit(1)
            }
        }
        Err(e) => {
            eprintln!("{e}");
            process::exit(1)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    fn test_match(input: &str, pattern: &str, expected: bool) {
        let res = match_pattern(input, pattern).unwrap();
        assert_eq!(res, expected, "input: {}, pattern: {}", input, pattern);
    }

    #[test]
    fn literal_pattern() {
        test_match("abc", "abc", true);
        test_match("abcd", "abc", true);
        test_match("ab", "abc", false);
        test_match("abce", "abc", true);
        test_match("uvwxyzabde", "abc", false);
    }

    #[test]
    fn digit_pattern() {
        test_match("1", r"\d", true);
        test_match("123", r"\d", true);
        test_match("a", r"\d", false);
        test_match(" ", r"\d", false);
        test_match("apple", r"\d", false);
    }

    #[test]
    fn alphanumeric_pattern() {
        test_match("x apple", r"\w", true);
        test_match("$!?", r"\w", false);
    }

    #[test]
    fn group_pattern() {
        test_match("x apple", "[abc]", true);
        test_match("x apple", "[^abc]", true);
        test_match("1 apple", r"\d apple", true);
        test_match("x apple", r"\d apple", false);
    }

    #[test]
    fn combinations_pattern() {
        test_match("sally has 124 apples", r"\d\d\d apples", true);
        test_match("sally has 12 apples", r"\d\d\d apples", false);
        test_match("sally has 3 dogs", r"\d \w\w\ws", true);
        test_match("sally has 4 dogs", r"\d \w\w\ws", true);
        test_match("sally has 1 dog", r"\d \w\w\ws", false);
    }

    #[test]
    fn start_of_string_pattern() {
        test_match("abc", "^abc", true);
        test_match("abcd", "^abc", true);
        test_match("ab", "^abc", false);
        test_match("abce", "^abc", true);
        test_match("aabc", "^abc", false);
    }

    #[test]
    fn end_of_string_pattern() {
        test_match("abc", "abc$", true);
        test_match("abcd", "abc$", false);
        test_match("ab", "abc$", false);
        test_match("abce", "abc$", false);
        test_match("aabc", "abc$", true);
        test_match("aabc", "abc$", true);
    }

    #[test]
    fn one_or_more_pattern() {
        test_match("aaaaaa", "a+", true);
        test_match("caaaats", "ca+t", true);
        test_match("apple", "a+", true);
        test_match("SaaS", "a+", true);
        test_match("dog", "a+", false);
    }

    #[test]
    fn optional_pattern() {
        test_match("dogs", "dogs?", true);
        test_match("dog", "dogs?", true);
        test_match("cat", "dogs?", false);
        test_match("dog", "do?g", true);
        test_match("dag", "do?g", false);
        test_match("ac", "ab?c", true);
    }
}
