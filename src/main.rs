use anyhow::bail;
use std::env;
use std::io;
use std::process;
use std::str::Bytes;

fn match_pattern(input: &str, pattern: &str) -> anyhow::Result<bool> {
    let mut input = input.bytes();

    loop {
        if match_here(&mut input.clone(), pattern.as_bytes())? {
            return Ok(true);
        }
        if input.next().is_none() {
            return Ok(false);
        }
    }
}

fn match_here(input: &mut Bytes, mut pattern: &[u8]) -> anyhow::Result<bool> {
    if pattern.is_empty() {
        return Ok(true);
    };

    let Some(input_ch) = input.next() else {
        return Ok(false);
    };

    let pattern_ch = pattern[0];

    let (is_char_matches, skip_index) = if pattern_ch == b'\\' {
        (match_char_type(&input_ch, &pattern[1..]), 2)
    } else if pattern_ch == b'[' {
        match_char_group(&input_ch, &pattern[1..])?
    } else {
        (match_char(&input_ch, &pattern_ch), 1)
    };

    if is_char_matches {
        match_here(input, &pattern[skip_index..])
    } else {
        Ok(false)
    }
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
