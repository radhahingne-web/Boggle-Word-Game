//! Reading what the player types.

use std::io::{self, Write};

/// Show `message`, wait for the player to type a line and press Enter,
/// and return what they typed (without the Enter at the end).
///
/// Returns `None` if the input has ended (for example the player pressed
/// Ctrl-D / Ctrl-Z). The games treat that the same as quitting.
/// (`Option` means "there is either `Some` value, or `None`".)
pub fn prompt(message: &str) -> Option<String> {
    print!("{}", message);
    io::stdout().flush().ok(); // make sure the message appears before we wait

    let mut line = String::new();
    match io::stdin().read_line(&mut line) {
        Ok(0) => None, // 0 bytes read = nothing more to read
        Ok(_) => Some(line.trim().to_string()),
        Err(_) => None,
    }
}
