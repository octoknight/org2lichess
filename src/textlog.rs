use crate::types::*;
use chrono::Utc;
use std::fs::OpenOptions;
use std::io::prelude::*;

pub fn append_line_to(filename: &str, line: &str) -> Result<(), ErrorBox> {
    let mut file = OpenOptions::new().append(true).open(filename)?;

    writeln!(file, "[{}] {}", Utc::now(), line)?;
    Ok(())
}
