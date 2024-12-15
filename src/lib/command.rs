use std::str::FromStr;

use clap::Parser;

use crate::Pattern;

#[derive(Parser, Debug)]
#[command(about, author, version)]
/// Grep text by pattern from stdin
pub struct Command {
    /// Should match Regexp pattern
    #[arg(short = 'e', default_value_t = true)]
    regexp: bool,

    /// Pattern to grep from the input
    pattern: String,
}

impl Command {
    pub fn invoke(&self) -> anyhow::Result<i32> {
        let mut input_line = String::new();

        std::io::stdin().read_line(&mut input_line).unwrap();

        let input_line = input_line.trim_end();
        let mut pattern = Pattern::from_str(&self.pattern).unwrap();
        let res = pattern.match_line(input_line)?;

        if res {
            Ok(0)
        } else {
            Ok(1)
        }
    }
}
