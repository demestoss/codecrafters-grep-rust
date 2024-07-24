use std::env;
use std::io;
use std::process;
use std::str::Chars;

fn match_pattern(mut input: &str, pattern: &str) -> bool {
    let mut input = input.chars();
    let pattern = pattern.chars();

    while input.next().is_some() {
        if match_here(&mut input.clone(), &mut pattern.clone()) {
            return true;
        }
    }
    false
}

fn match_here(input: &mut Chars, pattern: &mut Chars) -> bool {
    let Some(pattern_ch) = pattern.next() else {
        return true;
    };

    let Some(input_ch) = input.next() else {
        return false;
    };

    let is_char_matches = if pattern_ch == '\\' {
        match_char_type(&input_ch, pattern)
    } else if pattern_ch == '[' {
        match_char_group(&input_ch, pattern)
    } else {
        match_char(&input_ch, &pattern_ch)
    };

    println!("{input:?} {is_char_matches} {pattern:?}");

    is_char_matches && match_here(input, pattern)
}

fn match_char_type(input_ch: &char, pattern: &mut Chars) -> bool {
    let Some(pattern_ch) = pattern.next() else {
        return false;
    };
    match pattern_ch {
        'd' => input_ch.is_digit(10),
        'w' => input_ch.is_alphanumeric(),
        _ => false,
    }
}

fn match_char_group(input_ch: &char, pattern: &mut Chars) -> bool {
    todo!()
}

fn match_char(input_ch: &char, pattern_ch: &char) -> bool {
    input_ch == pattern_ch
}

// fn match_here(input_line: &str, pattern: &str) -> bool {
//     if pattern.chars().count() == 1 {
//         input_line.contains(pattern)
//     } else if pattern == r"\d" {
//         input_line.contains(|c: char| c.is_digit(10))
//     } else if pattern == r"\w" {
//         input_line.contains(|c: char| c.is_alphanumeric())
//     } else if pattern.starts_with("[^") && pattern.ends_with("]") {
//         let pattern = pattern.trim_matches(&['[', ']']);
//         let pattern = &pattern[1..];
//         input_line.contains(|c: char| !pattern.contains(c))
//     } else if pattern.starts_with("[") && pattern.ends_with("]") {
//         let pattern = pattern.trim_matches(&['[', ']']);
//         input_line.contains(|c: char| pattern.contains(c))
//     } else {
//         panic!("Unhandled pattern: {}", pattern)
//     }
// }

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
