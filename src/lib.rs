//! [crate] A simple and naive implementation of the FTP protocol.
//! This library doesn't support all FTP commands. See [README.md].
//! This library doesn't provide encripted data transmission.
//! # Example:
//! ```no_run
//! use simpleftp::FtpClient;
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

use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::net::Shutdown;
use std::net::TcpStream;
use std::net::ToSocketAddrs;

use std::io::ErrorKind;
#[allow(dead_code)]

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

///A simple FTP Client implementation.
/// Offers FTP commands to:
/// * Login
/// * Copy files
/// * Rename files
/// * Delete files/directories
/// * List files
///
pub struct FtpClient {
    reader: BufReader<TcpStream>,
}

impl FtpClient {
    /// Open a FTP connection
    ///
    /// # Arguments
    /// `address`     Server address to connect
    ///
    /// # Example
    /// ```no_run
    /// use simpleftp::Result;
    /// use simpleftp::*;
    ///
    /// fn main() -> Result<()> {
    ///    let mut client = FtpClient::connect("127.0.0.1:21")?;
    ///    client.login("user", "password")?;
    ///    client.logout()?;
    ///
    ///    Ok(())
    ///}
    ///```
    pub fn connect(address: impl ToSocketAddrs) -> Result<Self> {
        let reader = BufReader::new(TcpStream::connect(address)?);
        let mut client = FtpClient { reader };

        if client.parse_response()?.code != SERVICE_READY {
            return Err(FtpError::ConnectionError(
                "Server not ready for conenctions".into(),
            ));
        }
        Ok(client)
    }

    /// Perform Login to server.
    /// # Arguments
    /// `username `   username for login
    /// `password`    password for given us er
    ///
    /// # Errors
    /// May return LoginError if login is not successful.
    ///
    /// # Example
    /// ```no_run
    /// use simpleftp::Result;
    /// use simpleftp::*;
    /// fn main() -> simpleftp::Result<()>{
    ///     let mut client = FtpClient::connect("127.0.0.1:21")?;
    ///     client.login("user", "password")?;
    ///     client.logout()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn login(&mut self, username: &str, password: &str) -> Result<()> {
        // send username
        let response = self.write_cmd(format!("USER {}", username))?;

        if response.code != NEED_PASSWORD && response.code != LOGGED_IN {
            return Err(FtpError::LoginError(format!(
                "Could not authenticate: {}",
                response.code
            )));
        }

        // send password
        let response = self.write_cmd(format!("PASS {}", password))?;
        if response.code != LOGGED_IN {
            return Err(FtpError::LoginError(format!(
                "Invalid username/password combination: {}",
                response.code
            )));
        }

        Ok(())
    }

    /// Write a command to the server
    /// # Arguments
    /// `command`     text command to be sent to server
    /// # Errors
    /// Errors when failing to write to server or to parse a response.
    fn write_cmd(&mut self, command: impl AsRef<str>) -> Result<Response> {
        self.reader
            .get_mut()
            .write_all(format!("{}\r\n", command.as_ref()).as_bytes())?;
        self.parse_response()
    }

    /// Provide user account after login
    /// # Arguments
    /// account string representing the user account for login
    ///
    /// # Errors
    /// Errors when failing to write to server or to parse response.
    pub fn account(&mut self, account: impl AsRef<str>) -> Result<()> {
        match self.write_cmd(format!("ACCT {}", account.as_ref()))?.code {
            LOGGED_IN => Ok(()),
            _ => Err(FtpError::LoginError("Invalid account information".into())),
        }
    }

    /// Retrieve a file from the server.
    ///
    /// # Arguments
    /// `file`    Name of the file (includes path) on the server to be retrieved
    /// `dest`    Writer destination to dump data setn from server
    ///
    /// # Examples
    /// ```no_run
    /// use simpleftp::*;
    /// use simpleftp::Result;
    /// fn main() -> Result<()> {
    ///     let mut client = FtpClient::connect("127.0.0.1:21")?;
    ///     client.login("user", "passowrd")?;
    ///
    ///     let mut destination = std::fs::File::open("log.txt").expect("Opening file");
    ///     client.get("/home/will/code/log.txt", &mut destination)?;
    ///     client.logout()?;
    ///     Ok(())
    /// }
    ///
    /// ```
    /// # Errors
    /// Errors when failing to write to server or to parse response or due to connection problems.
    pub fn get(&mut self, file: impl AsRef<str>, dest: &mut impl Write) -> Result<()> {
        let mut stream = self.pasv()?;
        let response = self.write_cmd(format!("RETR {}", file.as_ref()))?;
        if response.code != FILE_OK && response.code != ALREADY_OPEN {
            return Err(FtpError::CommandError(
                "Could not process file retrieve".into(),
            ));
        }
        std::io::copy(&mut stream, dest)?;
        #[cfg(feature = "debug")]
        println!("Closing connection");
        match self.parse_response()?.code {
            CLOSING_DATA_CONNECTION => Ok(()),
            _ => Err(FtpError::ConnectionError("Error closing connection".into())),
        }
    }

    /// Sends a file to the server.
    ///
    /// # Arguments
    /// `file`    Name of the file (includes path) on the server to be retrieved
    /// `source`  Reader stream containing data to send to server
    ///
    /// # Examples
    /// ```no_run
    ///  use simpleftp::FtpClient;
    ///  use simpleftp::Result;
    ///
    /// fn main() -> Result<()> {
    ///     let mut client = FtpClient::connect("127.0.0.1:21")?;
    ///     client.login("user", "passowrd/")?;
    ///
    ///     let mut source = std::fs::File::create("log.txt").expect("Opening file");
    ///     client.put("/home/will/code/log.txt", &mut source)?;
    ///     client.logout()?;
    ///     Ok(())
    /// }
    /// ```
    /// # Errors
    /// Errors when failing to write to server or to parse response or due to connection problems.
    /// May also fail when reading from the source stream.
    pub fn put(&mut self, file: impl AsRef<str>, source: &mut impl Read) -> Result<()> {
        self.store_cmd(file, source, false)?;
        Ok(())
    }

    /// Sends a file to the server and stories in a unique location under current directory.
    ///
    /// # Arguments
    /// `file`    Name of the file (includes path) on the server to be retrieved
    /// `source`  Reader stream containing data to send to server
    ///
    ///  # Returns
    ///  A result string with name of the file created
    ///
    /// # Examples
    /// ```no_run
    ///  use simpleftp::FtpClient;
    ///  use simpleftp::Result;
    ///
    /// fn main() -> Result<()> {
    ///     let mut client = FtpClient::connect("127.0.0.1:21").unwrap();
    ///     client.login("user", "passowrd/").unwrap();
    ///
    ///     let mut source = std::fs::File::create("log.txt").expect("Opening file");
    ///     client.put_unique(&mut source).unwrap();
    ///     client.logout().unwrap();
    ///     Ok(())
    /// }
    /// ```
    /// # Errors
    /// Errors when failing to write to server or to parse response or due to connection problems.
    /// May also fail when reading from the source stream.
    pub fn put_unique(&mut self, source: &mut impl Read) -> Result<String> {
        self.store_cmd("", source, true)
    }

    /// Sends a file to the server. If file exists, append to it.
    ///
    /// # Arguments
    /// `file`    Name of the file (includes path) on the server to be retrieved
    /// `source`  Reader stream containing data to send to server
    ///
    /// # Examples
    /// ```no_run
    ///  use simpleftp::FtpClient;
    ///  use simpleftp::Result;
    ///
    /// fn main() -> Result<()> {
    ///     let mut client = FtpClient::connect("127.0.0.1:21").unwrap();
    ///     client.login("user", "passowrd/").unwrap();
    ///
    ///     let mut source = std::fs::File::create("log.txt").expect("Opening file");
    ///     client.append("/home/will/log.txt/", &mut source).unwrap();
    ///     client.logout().unwrap();
    ///     Ok(())
    /// }
    /// ```
    /// # Errors
    /// Errors when failing to write to server or to parse response or due to connection problems.
    /// May also fail when reading from the source stream.
    pub fn append(&mut self, file: impl AsRef<str>, source: &mut impl Read) -> Result<()> {
        self.store_cmd(file.as_ref(), source, false)?;
        Ok(())
    }
    fn store_cmd(
        &mut self,
        file: impl AsRef<str>,
        source: &mut impl Read,
        unique: bool,
    ) -> Result<String> {
        let mut stream = self.pasv()?;
        let response = if unique {
            self.write_cmd(format!("STOU {}", file.as_ref()))?
        } else {
            self.write_cmd(format!("STOR {}", file.as_ref()))?
        };

        if response.code != FILE_OK {
            return Err(FtpError::CommandError("Could not process file STOR".into()));
        }
        #[cfg(feature = "debug")]
        println!("Copying file:{}", file.as_ref());

        std::io::copy(source, &mut stream)?;

        #[cfg(feature = "debug")]
        println!("Closing connection");

        // close data connection
        stream.shutdown(Shutdown::Both)?;
        match self.parse_response()?.code {
            CLOSING_DATA_CONNECTION => Ok(response.message),
            _ => Err(FtpError::ConnectionError("Error closing connection".into())),
        }
    }

    /// Sends a NO OPERATION command
    ///
    /// # Errors
    /// When the connection to server fails or when the server provides invalid response.
    pub fn noop(&mut self) -> Result<()> {
        match self.write_cmd("NOOP")?.code {
            COMMAND_OK => Ok(()),
            _ => Err(FtpError::CommandError("failed command".into())),
        }
    }

    /// Rename a file on the server
    /// # Arguments
    /// `from`  current file name on server
    /// `to`    new file name to be given to server
    ///
    /// # Examples
    /// ```no_run
    /// # use simpleftp::FtpClient;
    /// # fn main() -> simpleftp::Result<()> {
    /// # use simpleftp::FtpClient;
    /// let mut client = FtpClient::connect("127.0.0.1:21")?;
    /// client.login("user", "password").unwrap();
    /// client.rename(
    ///     "/home/will/code/log.txt",
    ///     "/home/will/code/text.log",
    /// )?;
    /// client.logout()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Due to connection errors with the server, incorrect filenames or server response.
    pub fn rename(&mut self, from: impl AsRef<str>, to: impl AsRef<str>) -> Result<()> {
        let response = self.write_cmd(format!("RNFR {}", from.as_ref()))?;
        if response.code != FILE_ACTION_PENDING {
            return Err(FtpError::CommandError(format!(
                "Could not rename file: {}",
                response.code
            )));
        }

        let response = self.write_cmd(format!("RNTO {}", to.as_ref()))?;
        if response.code != FILE_ACTION_OK {
            return Err(FtpError::CommandError(format!(
                "Could not rename file: {}",
                response.code
            )));
        }
        Ok(())
    }

    /// Deletes a file from the server
    ///
    /// # Arguments
    /// `file`      Filename to be deleted on the server side
    ///
    /// # Examples
    /// ```no_run
    /// # use simpleftp::FtpClient;
    /// let mut client = FtpClient::connect("127.0.0.1:21").unwrap();
    /// client.login("user", "password").unwrap();
    /// client.delete("/home/will/code/log.txt").unwrap();
    /// client.logout().unwrap();
    /// ```
    pub fn delete(&mut self, file: impl AsRef<str>) -> Result<()> {
        let response = self.write_cmd(format!("DELE {}", file.as_ref()))?;
        if response.code != FILE_ACTION_OK {
            return Err(FtpError::CommandError(format!(
                "Could not delete file: {}",
                response.code
            )));
        }
        Ok(())
    }

    /// Retrieve data connection offered from the server
    ///  in the form of a TCP stream.
    ///
    /// # Errors
    /// If the connection cannot be established or if the server refuses.
    pub fn pasv(&mut self) -> Result<TcpStream> {
        let response = self.write_cmd("PASV")?;
        let code = response.code;
        if code != PASSIVE_MODE && code != ALREADY_OPEN {
            return Err(FtpError::ResponseError(format!(
                "Invalid response code from server: {}",
                code
            )));
        }
        let address = Self::extract_pasv_address(&response.message)?;

        #[cfg(feature = "debug")]
        println!("{}", address);

        let connection = TcpStream::connect(address)?;
        Ok(connection)
    }

    /// Get a list of files in the directory. Including file information.
    ///
    /// # Arguments
    /// `dir`   directory to git list udner
    ///
    /// # Errors
    /// Errors on connection failure or improper response from server
    pub fn list(&mut self, dir: &str) -> Result<Vec<String>> {
        self.list_cmd(dir, false)
    }

    /// Get a list of files in the directory. Names only.
    ///
    /// # Arguments
    /// `dir`   directory to git list udner
    ///
    /// # Errors
    /// Errors on connection failure or improper response from server
    pub fn name_list(&mut self, dir: &str) -> Result<Vec<String>> {
        self.list_cmd(dir, true)
    }

    /// Get a list of files in the directory. N
    ///
    /// # Arguments
    /// `dir`   directory to git list udner
    /// `named` if true get only names with no further information
    /// # Errors
    /// Errors on connection failure or improper response from server
    fn list_cmd(&mut self, dir: &str, named: bool) -> Result<Vec<String>> {
        let datacon = self.pasv()?;
        let response = if named {
            self.write_cmd(format!("NLST {}", dir))?
        } else {
            self.write_cmd(format!("LIST {}", dir))?
        };
        if response.code != COMMAND_OK && response.code != ALREADY_OPEN && response.code != FILE_OK
        {
            return Err(FtpError::CommandError(response.message));
        }
        let line_reader = BufReader::new(datacon);
        let file_list = line_reader
            .lines()
            .filter(|item| item.is_ok())
            .map(|item| item.unwrap())
            .collect();

        #[cfg(feature = "debug")]
        println!("Closing connection");
        match self.parse_response()?.code {
            CLOSING_DATA_CONNECTION => Ok(file_list),
            _ => Err(FtpError::ConnectionError("Error closing connection".into())),
        }
    }

    /// Change working directory on server side.
    ///
    /// # Arguments
    /// `dir`   Directory to be set as current working directory
    ///
    /// # Errors
    /// Errors on connection failure or inexisting directory.
    pub fn change_dir(&mut self, dir: impl AsRef<str>) -> Result<()> {
        let response = self.write_cmd(format!("CWD {}", dir.as_ref()))?;
        match response.code {
            COMMAND_OK | FILE_ACTION_OK => Ok(()),
            _other => Err(FtpError::FileError(format!(
                "Invalid response {}",
                response.message
            ))),
        }
    }

    /// Create directory on server side.
    ///
    /// # Arguments
    /// `dir`   directoryto created
    pub fn makedir(&mut self, dir: impl AsRef<str>) -> Result<()> {
        let response = self.write_cmd(format!("MKD {}", dir.as_ref()))?;
        match response.code {
            DIRECTORY_ALREADY_EXISTS | PATH_CREATED => Ok(()),
            _other => Err(FtpError::FileError(format!(
                "Invalid response {}",
                response.message
            ))),
        }
    }

    /// Remove directory on server side.
    ///
    /// # Arguments
    /// `dir`   directoryto removed
    pub fn remove_dir(&mut self, dir: impl AsRef<str>) -> Result<()> {
        let response = self.write_cmd(format!("RMD {}", dir.as_ref()))?;
        match response.code {
            FILE_ACTION_OK => Ok(()),
            _other => Err(FtpError::FileError(format!(
                "Invalid response {}",
                response.message
            ))),
        }
    }

    /// Go to parent directory on server side.
    pub fn change_dir_up(&mut self) -> Result<()> {
        let response = self.write_cmd("CDUP")?;
        match response.code {
            COMMAND_OK | FILE_ACTION_OK => Ok(()),
            _other => Err(FtpError::CommandError(format!(
                "Invalid response {}",
                response.message
            ))),
        }
    }

    /// Print  directory on server side.
    pub fn pwd(&mut self) -> Result<String> {
        let response = self.write_cmd("PWD")?;
        match response.code {
            PATH_CREATED => Ok(response.message),
            _other => Err(FtpError::CommandError(format!(
                "Invalid response {}",
                response.message
            ))),
        }
    }

    /// Close current data connection
    pub fn abort(&mut self) -> Result<()> {
        let response = self.write_cmd("ABOR")?;
        match response.code {
            CLOSING_DATA_CONNECTION => Ok(()),
            _other => Err(FtpError::CommandError(format!(
                "Invalid response {}",
                response.message
            ))),
        }
    }

    /// Get status of directory
    ///
    /// # Arguments
    /// `path`  path of item to get status
    pub fn status(&mut self, path: impl AsRef<str>) -> Result<String> {
        let response = self.write_cmd(format!("STAT {}", path.as_ref()))?;
        match response.code {
            SYSTEM | FILE | DIRECTORY => Ok(response.message),
            _other => Err(FtpError::CommandError(format!(
                "Invalid response {}",
                response.message
            ))),
        }
    }

    /// Get server information
    pub fn system(&mut self) -> Result<String> {
        let response = self.write_cmd("SYST")?;
        match response.code {
            SYSTEM | NAME_SYSTEM => Ok(response.message),
            _other => Err(FtpError::CommandError(format!(
                "Invalid response {}",
                response.message
            ))),
        }
    }

    /// Get help for given item
    ///
    /// # Arguments
    /// `item`  item to retrieve information for
    ///
    /// # Examples:
    /// ```no_run
    /// # use simpleftp::FtpClient;
    /// let mut client = FtpClient::connect("127.0.0.1:21").unwrap();
    /// client.login("user", "password").unwrap();
    /// client.help("SITE").unwrap();
    /// client.logout().unwrap();
    /// ```
    pub fn help(&mut self, item: impl AsRef<str>) -> Result<String> {
        let response = self.write_cmd(format!("HELP {}", item.as_ref()))?;
        match response.code {
            HELP_MESSAGE | FILE => Ok(response.message),
            _other => Err(FtpError::CommandError(format!(
                "Invalid response {}",
                response.message
            ))),
        }
    }

    /// Allocate space for a file transfer
    ///
    /// # Arguments
    /// `size`  path of item to get status
    pub fn allocate(&mut self, size: usize) -> Result<()> {
        let response = self.write_cmd(format!("ALLO {}", size))?;
        match response.code {
            COMMAND_OK => Ok(()),
            _other => Err(FtpError::CommandError(format!(
                "Invalid response {}",
                response.message
            ))),
        }
    }

    /// Mount a different filesystem on the server.
    ///
    /// # Arguments
    /// `pathname`   Path to the device to mount
    ///
    /// # Errors
    /// Errors on connection failure or inexisting pathname.
    pub fn mount(&mut self, pathname: impl AsRef<str>) -> Result<()> {
        let response = self.write_cmd(format!("SMNT {}", pathname.as_ref()))?;
        match response.code {
            COMMAND_OK | FILE_ACTION_OK => Ok(()),
            _other => Err(FtpError::FileError(format!(
                "Invalid response {}",
                response.message
            ))),
        }
    }

    /// Disconnect from the server
    ///
    /// # Errors
    /// On the strange circumstances the server refuses the logout operation
    /// or does not recognize the command.
    pub fn logout(&mut self) -> Result<()> {
        let response = self.write_cmd("QUIT")?;
        match response.code {
            SERVICE_CLOSING => Ok(()),
            _other => Err(FtpError::FileError(format!(
                "Invalid response {}",
                response.message
            ))),
        }
    }

    /// Set transfer mode to binary.
    /// # Errors
    /// On connection failure or when type not suported by server
    pub fn binary_mode(&mut self) -> Result<()> {
        let response = self.write_cmd("TYPE I")?;
        match response.code {
            COMMAND_OK => Ok(()),
            _other => Err(FtpError::FileError(format!(
                "Invalid response {}",
                response.message
            ))),
        }
    }

    /// Set transfer mode to ASCII.
    /// # Errors
    /// On connection failure or when type not suported by server
    pub fn ascii_mode(&mut self) -> Result<()> {
        let response = self.write_cmd("TYPE A")?;
        match response.code {
            COMMAND_OK => Ok(()),
            _other => Err(FtpError::FileError(format!(
                "Invalid response {}",
                response.message
            ))),
        }
    }

    /// Reads a response and returns the server's response
    fn parse_response(&mut self) -> Result<Response> {
        let mut response = String::new();
        self.reader
            .read_line(&mut response)
            .map_err(|_| FtpError::ResponseError("Could not read server response".into()))?;
        #[cfg(feature = "debug")]
        print!("Parsing: {}", response);

        if response.len() < 5 {
            return Err(FtpError::ResponseError(format!(
                "Invalid response code form server: {}",
                response
            )));
        }
        let code: usize = response[0..3].parse().map_err(|_| {
            FtpError::ResponseError(format!("Invalid response code form server: {}", response))
        })?;

        // multiline response
        if response[0..4].contains('-') {
            let mut new_line = String::new();
            while !new_line.starts_with(&response[0..3]) {
                new_line.clear();
                self.reader.read_line(&mut new_line)?;
                response.push_str(&new_line[..]);
                #[cfg(feature = "debug")]
                println!("multi-line  {}", new_line);
            }
        }

        Ok(Response {
            code,
            message: response[3..].to_string(),
        })
    }

    // Helper method to extract the TCP connection address common on PASV and PORT responses
    fn extract_pasv_address(response: &str) -> Result<String> {
        let ipinfo = response.chars().filter(|c| *c==',' || c.is_numeric()).collect::<String>();
        let tokens = ipinfo.split(",").map(|tok| tok.trim().to_string()).collect::<Vec<String>>();
        let numbers = tokens.iter().filter(|tok| tok.parse::<u32>().is_ok()).map(|tok| tok.parse::<u32>().unwrap()).collect::<Vec<u32>>();

        let format_error =
            FtpError::ResponseError(format!("Invalid PASV response from server: {}", response));

        if numbers.len() < 6 {
            Err(format_error.clone())
        } else {
            let port_lower: u32 = numbers[5];
            let port_upper: u32 = numbers[4];

            let full_port = port_upper * 256 + port_lower;
            Ok(format!(
                "{}.{}.{}.{}:{}",
                &numbers[0], &numbers[1], &numbers[2], &numbers[3], full_port
            ))
        }
    }
}
