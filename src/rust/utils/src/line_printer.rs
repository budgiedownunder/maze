use std::io::{self};

/// Represents a line printer interface
pub trait LinePrinter {
    /// Prints a given line of text
    /// # Arguments
    /// * `line` - Text to print
    ///
    /// # Returns
    ///
    /// This function should return an error if the print operation does not succeed
    fn print_line(&mut self, line: &str) -> Result<(), io::Error>;
}
