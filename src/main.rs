mod match_pattern;
mod pattern;

use crate::match_pattern::match_pattern;
use std::env;
use std::io;
use std::process;

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
    use crate::match_pattern::match_pattern;
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

    #[test]
    fn exact_quantifier_pattern() {
        test_match("dogs", "dog{1}s", true);
        test_match("doggs", "dog{2}s", true);
        test_match("dogggs", "dog{2}s", false);
        test_match("doggs", "dog{1}s", false);
    }

    #[test]
    fn between_quantifier_pattern() {
        test_match("dog", "dog{1,3}", true);
        test_match("dogg", "dog{1,3}", true);
        test_match("doggggs", "dog{1,3}s", false);
    }

    #[test]
    fn at_least_quantifier_pattern() {
        test_match("dog", "dog{2,}", false);
        test_match("dogg", "dog{2,}", true);
        test_match("doggggg", "dog{2,}", true);
    }

    #[test]
    fn whitespace_pattern() {
        test_match("do     g", r"do\sg", true);
        test_match("dog", r"do\s?g", true);
        test_match("do\tg", r"do\sg", true);
        test_match("do\t      g", r"do\sg", true);
    }
}
