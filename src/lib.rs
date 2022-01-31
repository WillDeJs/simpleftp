#[allow(dead_code)]
pub mod client;

#[derive(Debug, Clone)]
pub enum FtpError {
    LoginError(String),
    ConnectionError(String),
    FileError(String),
    CommandError(String),
    ResponseError(String),
}
pub type Result<T> = std::result::Result<T, FtpError>;

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
    pub const COMMAND_OK_: usize = 200;
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
    pub const FILE_ACTION_ENDING: usize = 350;
    pub const FILE_ACTION_NOT_TAKEN: usize = 450;
    pub const LOCAL_ERROR: usize = 451;
    pub const PARAMETER_ERROR: usize = 501;
    pub const INSUFFICIENT_STORAGE: usize = 452;
    pub const FILE_NOT_AVAILABLE: usize = 550;
    pub const PAGE_TYPE_UNKNOWN: usize = 551;
    pub const FILE_ACTION_ABORTED: usize = 552;
    pub const FILE_NAME_NOT_ALLOWED: usize = 553;
}
