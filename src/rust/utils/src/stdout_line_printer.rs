use crate::LinePrinter;
use std::io::{self};

/// Represents a line printer for targetting stdout
pub struct StdoutLinePrinter {}

impl StdoutLinePrinter {
    /// Creates a new stdout line printer instance
    ///
    /// # Returns
    ///
    /// A new stdout line printer instance
    pub fn new() -> StdoutLinePrinter {
        StdoutLinePrinter {}
    }
}

impl Default for StdoutLinePrinter {
    fn default() -> Self {
        Self::new()
    }
}

impl LinePrinter for StdoutLinePrinter {
    /// Prints the given text to stdout followed by a newline
    /// # Arguments
    /// * `line` - Text to print
    ///
    /// # Returns
    ///
    /// This function will return an error if the print operation did not succeed
    ///
    /// # Examples
    ///
    /// ```
    /// use utils::{LinePrinter, StdoutLinePrinter};
    /// let mut print_target = StdoutLinePrinter::new();
    /// print_target.print_line("First line of text");
    /// print_target.print_line("Second line of text");
    /// ```
    fn print_line(&mut self, line: &str) -> Result<(), io::Error> {
        println!("{}", line);
        Ok(())
    }
}
