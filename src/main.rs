mod pattern;

use crate::pattern::{CharToken, Pattern, PatternItem};
use std::env;
use std::io;
use std::process;
use std::str::{Bytes, FromStr};

fn match_pattern(input: &str, pattern: &str) -> anyhow::Result<bool> {
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

fn match_here(input: &mut Bytes, pattern: &mut Pattern) -> bool {
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

        if pattern_item.is_least_matched(match_times + 1) {
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
        test_match("banana", "[^anb]", false);
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

    #[test]
    fn wildcard_pattern() {
        test_match("dogs", "do.s", true);
        test_match("doqs", "do.?s", true);
        test_match("cats", "do.s", false);
        test_match("sddsddssas", ".+as", true);
        test_match("ddsdsaDdsds", ".+as?", true);
        test_match("mod.rs", "*.rs", true);
    }

    #[test]
    fn alteration_pattern() {
        test_match("dog", "(dog|cat)", true);
        test_match("cat", "(dog|cat)", true);
        test_match("apple", "(dog|cat)", false);
    }

    // #[test]
    // fn exact_quantifier_pattern() {
    //     test_match("dog", "dog{1}", true);
    //     test_match("dogg", "dog{1}", false);
    // }
    //
    // #[test]
    // fn between_quantifier_pattern() {
    //     test_match("dog", "dog{1,3}", true);
    //     test_match("dogg", "dog{1,3}", true);
    //     test_match("dogggg", "dog{1,3}", false);
    // }

    // #[test]
    // fn at_least_quantifier_pattern() {
    //     test_match("dog", "dog{2,}", false);
    //     test_match("dogg", "dog{2,}", true);
    //     test_match("doggggg", "dog{2,}", true);
    // }

    #[test]
    fn whitespace_pattern() {
        test_match("do     g", r"do\sg", true);
        test_match("dog", r"do\s?g", true);
        test_match("do\tg", r"do\sg", true);
        test_match("do\t      g", r"do\sg", true);
    }
}
