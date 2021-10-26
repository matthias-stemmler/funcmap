use std::{
    io::{self, BufRead, Write},
    process::{Command, Stdio},
    str,
};

use proc_macro2::TokenStream;

pub fn print(tokens: &TokenStream) {
    if let Err(err) = print_formatted(&tokens.to_string()) {
        eprintln!("[mapstruct] Failed to format debug output: {}", err);
    }
}

fn print_formatted(input: &str) -> io::Result<()> {
    let mut rustfmt = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    rustfmt.stdin.take().unwrap().write_all(input.as_bytes())?;

    let stdout = rustfmt.wait_with_output()?.stdout;

    let mut output = Vec::new();
    for line in stdout.lines() {
        writeln!(output, "[mapstruct] {}", line?)?;
    }

    io::stdout().write_all(&output)?;

    Ok(())
}
