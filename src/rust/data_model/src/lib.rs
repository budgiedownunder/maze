// Re-export modules
mod email_audit;
mod error;
mod maze;
mod maze_cell_state;
mod maze_definition;
mod maze_point;
mod oauth_identity;
mod one_time_token;
mod user;
mod user_email;
mod user_login;
mod wrappers;

// Re-export traits and structs
pub use email_audit::{
    AuditOutcome, EMAIL_AUDIT_ERROR_MESSAGE_MAX_CHARS, ERROR_MESSAGE_TRUNCATION_MARKER,
    EmailAuditEntry, truncate_email_audit_error_message,
};
pub use error::{Error, UserValidationError};
pub use maze_definition::MazeDefinition;
pub use maze::Maze;
pub use maze_cell_state::MazeCellState;
pub use maze_point::MazePoint;
pub use oauth_identity::OAuthIdentity;
pub use one_time_token::{OneTimeToken, TokenPurpose};
pub use user::{is_valid_email_format, User};
pub use user_email::UserEmail;
pub use user_login::UserLogin;
