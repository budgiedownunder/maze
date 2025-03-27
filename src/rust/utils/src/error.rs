use std::io::ErrorKind;

/// Converts an ErrorKind to a string
pub fn io_error_kind_to_string(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::NotFound => "File or directory not found",
        ErrorKind::PermissionDenied => "Permission denied",
        ErrorKind::ConnectionRefused => "Connection refused",
        ErrorKind::ConnectionReset => "Connection reset by remote server",
        ErrorKind::ConnectionAborted => "Connection aborted (terminated) by remote server",
        ErrorKind::NotConnected => "Network operation failed because it is not connected yet",
        ErrorKind::AddrInUse => {
            "Socket address could not be bound because the address is already in use elsewhere"
        }
        ErrorKind::AddrNotAvailable => {
            "Nonexistent interface was requested or the requested address was not local"
        }
        ErrorKind::BrokenPipe => "Operation failed because pipe was closed",
        ErrorKind::AlreadyExists => "File already exists",
        ErrorKind::WouldBlock => "Operation would block",
        ErrorKind::InvalidInput => "Invalid input",
        ErrorKind::InvalidData => "Invalid data",
        ErrorKind::TimedOut => "Timeout expired, causing operation to be cancelled",
        ErrorKind::WriteZero => "Write operation could not be fully completed",
        ErrorKind::Interrupted => "Operation interrupted",
        ErrorKind::Unsupported => "Operation is unsupported on this platform",
        ErrorKind::UnexpectedEof => "Unexpected end of file",
        ErrorKind::OutOfMemory => "Insufficient memory",
        ErrorKind::Other => "Custom I/O error",
        _ => "Unknown I/O error",
    }
}