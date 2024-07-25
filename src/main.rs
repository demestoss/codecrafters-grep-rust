use anyhow::bail;
use std::env;
use std::io;
use std::process;
use std::str::Bytes;

fn match_pattern(input: &str, pattern: &str) -> anyhow::Result<bool> {
    let mut input = input.bytes();
    let pattern = pattern.as_bytes();

    if pattern[0] == b'^' {
        return match_here(&mut input, &pattern[1..]);
    }

    loop {
        if match_here(&mut input.clone(), pattern)? {
            return Ok(true);
        }
        if input.next().is_none() {
            return Ok(false);
        }
    }
}

fn match_here(input: &mut Bytes, pattern: &[u8]) -> anyhow::Result<bool> {
    if pattern.is_empty() {
        return Ok(true);
    };
    if is_end_line_pattern(&pattern) && input.len() == 0 {
        return Ok(true);
    }

    let Some(input_ch) = input.next() else {
        return Ok(false);
    };
    let pattern_ch = pattern[0];

    let (is_char_matches, skip_index) = if pattern_ch == b'\\' {
        (match_char_type(&input_ch, &pattern[1..]), 2)
    } else if pattern_ch == b'[' {
        match_char_group(&input_ch, &pattern[1..])?
    } else if is_next_char_plus(&pattern) {
        (match_one_or_more(input, &input_ch, &pattern_ch), 2)
    } else {
        (match_char(&input_ch, &pattern_ch), 1)
    };

    if is_char_matches {
        match_here(input, &pattern[skip_index..])
    } else {
        Ok(false)
    }
}

fn is_end_line_pattern(pattern: &[u8]) -> bool {
    pattern.len() == 1 && pattern[0] == b'$'
}

fn is_next_char_plus(pattern: &[u8]) -> bool {
    matches!(pattern.get(1), Some(b'+'))
}

fn match_one_or_more(input: &mut Bytes, input_ch: &u8, pattern_ch: &u8) -> bool {
    if !match_char(&input_ch, &pattern_ch) {
        return false;
    }
    let skip_chars_count = input
        .clone()
        .take_while(|&input_ch| match_char(&input_ch, pattern_ch))
        .count();
    for _ in 0..skip_chars_count {
        input.next();
    }
    true
}

fn match_char_type(input_ch: &u8, pattern: &[u8]) -> bool {
    if pattern.is_empty() {
        return false;
    }
    match pattern[0] {
        b'd' => input_ch.is_ascii_digit(),
        b'w' => input_ch.is_ascii_alphanumeric(),
        _ => false,
    }
}

fn match_char_group(input_ch: &u8, pattern: &[u8]) -> anyhow::Result<(bool, usize)> {
    let Some((end_index, _)) = pattern.iter().enumerate().find(|&(_index, i)| *i == b']') else {
        bail!("incorrect group pattern: no closing brace")
    };

    let group = &pattern[..end_index];
    if group.is_empty() {
        bail!("incorrect group pattern: empty group")
    }

    let matches = match_group_char(&input_ch, &group);
    Ok((matches, end_index + 2))
}

fn match_group_char(input_ch: &u8, group: &[u8]) -> bool {
    match group[0] {
        b'^' => !group.contains(input_ch),
        _ => group.contains(input_ch),
    }
}

fn match_char(input_ch: &u8, pattern_ch: &u8) -> bool {
    input_ch == pattern_ch
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
}
