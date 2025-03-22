// Re-export modules
pub mod error;
pub mod file;
mod line_printer;
mod stdout_line_printer;

// Re-export traits and structs
pub use line_printer::LinePrinter;
pub use stdout_line_printer::StdoutLinePrinter;
