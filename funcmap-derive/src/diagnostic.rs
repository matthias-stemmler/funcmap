use proc_macro2::TokenStream;
use std::io::{self, Write};
use std::process::{Command, Stdio};

pub fn print(tokens: &TokenStream) {
    if let Err(err) = print_formatted(&tokens.to_string()) {
        eprintln!("Failed to format debug output: {}", err);
    }
}

fn print_formatted(input: &str) -> io::Result<()> {
    let mut rustfmt = Command::new("rustfmt").stdin(Stdio::piped()).spawn()?;
    rustfmt.stdin.take().unwrap().write_all(input.as_bytes())?;
    rustfmt.wait()?;
    Ok(())
}
