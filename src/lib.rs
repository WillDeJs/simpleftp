//! [crate] A simple and naive implementation of the FTP protocol.
//! This library doesn't support all FTP commands. See [README.md].
//! This library doesn't provide encripted data transmission.
//! # Example:
//! ```no_run
//! use simpleftp::client::FtpClient;
//! use simpleftp::Result;
//!
//! fn main() -> Result<()> {
//!     // connect to server
//!     let mut client = FtpClient::connect("test.rebex.net:21")?;
//!     client.login("demo", "password")?;
//!
//!     // download file
//!     let mut readme = std::fs::File::create("readme.txt")?;
//!     client.get("/readme.txt", &mut readme)?;
//!
//!     // disconnect from server
//!     client.logout()?;
//!     Ok(())
//! }
//!```

use std::io::ErrorKind;
#[allow(dead_code)]
pub mod client;

/// A generic FTP representation enum
#[derive(Debug, Clone)]
pub enum FtpError {
    LoginError(String),
    ConnectionError(String),
    FileError(String),
    CommandError(String),
    ResponseError(String),
}
impl From<std::io::Error> for FtpError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            ErrorKind::NotFound => Self::FileError("IO resource not found".into()),
            ErrorKind::PermissionDenied => Self::FileError("IO resource permissions denied".into()),
            ErrorKind::ConnectionRefused
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::NotConnected => {
                Self::ConnectionError("IO resource connection failed".into())
            }
            ErrorKind::TimedOut => Self::ConnectionError("connection timed out".into()),
            _ => Self::FileError("Error accessing file/reader/writer".into()),
        }
    }
}

impl std::fmt::Display for FtpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FtpError::LoginError(error) => write!(f, "Login Error {}", error),
            FtpError::ConnectionError(error) => write!(f, "Connection Error: {}", error),
            FtpError::FileError(error) => write!(f, "File Error: {}", error),
            FtpError::CommandError(error) => write!(f, "Command Error: {}", error),
            FtpError::ResponseError(error) => write!(f, "Response Error: {}", error),
        }
    }
}
/// Generic Result type defaults to Result<T, FtpError>
pub type Result<T> = std::result::Result<T, FtpError>;

#[doc(hidden)]
/// FTP Response Implementation
#[allow(dead_code)]
pub(crate) struct Response {
    code: usize,
    message: String,
}
#[allow(dead_code)]
pub(crate) mod status {

    /* Response codes definitions */

    pub const RESTART_MARKER: usize = 110;

    // Status messages
    pub const SYSTEM: usize = 211;
    pub const DIRECTORY: usize = 212;
    pub const FILE: usize = 213;
    pub const HELP_MESSAGE: usize = 214;
    pub const NAME_SYSTEM: usize = 215;

    // Command related messages
    pub const COMMAND_OK: usize = 200;
    pub const COMMAND_NOT_IMPLEMENTED: usize = 202;
    pub const UNKNOWN_COMMAND: usize = 500;
    pub const COMMAND_UNIMPLEMENTED: usize = 502;
    pub const BAD_COMMAND_SEQUENCE: usize = 503;
    pub const BAD_PARAMETER_FOR_COMMAND: usize = 504;

    // Related to service
    pub const READY_MINUTE: usize = 120;
    pub const SERVICE_READY: usize = 220;
    pub const SERVICE_CLOSING: usize = 221;
    pub const NOT_AVAILABLE: usize = 421;

    // Data connection
    pub const ALREADY_OPEN: usize = 125;
    pub const DATA_CONNECTION_OPEN: usize = 225;
    pub const CLOSING_DATA_CONNECTION: usize = 226;
    pub const CANNOT_OPEN_DATA_CONNECTION: usize = 425;
    pub const TRANSFER_ABORTED: usize = 426;
    pub const PASSIVE_MODE: usize = 227;

    // Loging messages
    pub const LOGGED_IN: usize = 230;
    pub const NOT_LOGGED_IN: usize = 530;
    pub const NEED_PASSWORD: usize = 331;
    pub const NEED_ACCOUNT: usize = 332;
    pub const ACCOUNT_NEEDED_FOR_FILE_CREATION: usize = 532;

    //File actions
    pub const FILE_OK: usize = 150;
    pub const FILE_ACTION_OK: usize = 250;
    pub const PATH_CREATED: usize = 257;
    pub const FILE_ACTION_PENDING: usize = 350;
    pub const FILE_ACTION_NOT_TAKEN: usize = 450;
    pub const LOCAL_ERROR: usize = 451;
    pub const PARAMETER_ERROR: usize = 501;
    pub const INSUFFICIENT_STORAGE: usize = 452;
    pub const FILE_NOT_AVAILABLE: usize = 550;
    pub const PAGE_TYPE_UNKNOWN: usize = 551;
    pub const FILE_ACTION_ABORTED: usize = 552;
    pub const FILE_NAME_NOT_ALLOWED: usize = 553;
    pub const DIRECTORY_ALREADY_EXISTS: usize = 521;
}
