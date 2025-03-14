use std::io;
use std::io::{IsTerminal, Write, stdout};

use crossterm::ExecutableCommand;
use crossterm::cursor::{RestorePosition, SavePosition};
use crossterm::terminal::{Clear, ClearType};
use strum::{AsRefStr, Display};
use yansi::{Color, Paint, Painted, Style};

//
// Status line
//

/// A status line that is only active if stdout is a terminal.
/// Clears the status line when dropped.
pub struct StatusLine {
    show_output: bool,
}

impl Drop for StatusLine {
    /// Clears the status line when dropped.
    fn drop(&mut self) {
        if self.show_output {
            Self::clear_last_line().ok();
        }
    }
}

impl StatusLine {
    /// Creates a new `StatusLine` message with the given tag and message.
    /// The message is cleared when dropped.
    pub fn new(tag: &str, msg: &str) -> Self {
        let show_output = stdout().is_terminal();
        if show_output {
            Self::print_updatable_line(tag, msg, MessageType::Info).ok();
        }
        Self { show_output }
    }

    /// Creates a new `StatusLine` that does not output anything.
    pub const fn new_silent() -> Self {
        Self { show_output: false }
    }

    /// Updates the status line with the given tag and message.
    pub fn update(&self, tag: &str, msg: &str) {
        if self.show_output {
            Self::clear_last_line().ok();
            Self::print_updatable_line(tag, msg, MessageType::Info).ok();
        }
    }

    /// Prints a status line with the given tag and message that is cleared.
    fn print_updatable_line(tag: &str, msg: &str, status: MessageType) -> io::Result<()> {
        stdout().execute(SavePosition).and_then(|o| {
            print!("{:>12} {}", MessageType::format_text(tag, status), msg);
            o.execute(RestorePosition)?.flush()
        })
    }

    fn clear_last_line() -> io::Result<()> {
        stdout()
            .execute(Clear(ClearType::FromCursorDown))
            .map(|_| ())
    }
}

//
// Message type
//

/// The type of message to display.
#[allow(dead_code)]
#[derive(Display, AsRefStr, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    None,
    Ok,
    Error,
    Warning,
    Info,
}

impl MessageType {
    /// Applies a style to the given string based on the message type.
    pub fn format_text(s: &str, message_type: Self) -> Painted<&str> {
        let color = match message_type {
            Self::None => Style::new().bold(),
            Self::Ok => Color::Green.bold(),
            Self::Error => Color::Red.bold(),
            Self::Warning => Color::Yellow.bold(),
            Self::Info => Color::Blue.bold(),
        };
        s.paint(color)
    }

    /// Prints a status line with the given tag and message.
    pub fn print_line(tag: &str, msg: &str, message_type: Self) {
        println!("{:>12} {}", Self::format_text(tag, message_type), msg);
    }
}
