use chrono::Utc;
use std::fs::OpenOptions;
use std::io::prelude::*;

pub fn append_line_to(filename: &str, line: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = OpenOptions::new().append(true).open(filename)?;

    writeln!(file, "[{}] {}", Utc::now(), line)?;
    Ok(())
}
