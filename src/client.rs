use super::status::*;
use super::*;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::net::ToSocketAddrs;

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
    /// ```no_compile
    /// fn main() -> Result<()> {
    ///    let mut client = FtpClient::connect("127.0.0.1:21")?;
    ///    client.login("user", "password")?;
    ///    client.quit()?;
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
    /// ```no_compile
    /// fn main() -> Result<()>{
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
    fn write_cmd(&mut self, command: String) -> Result<Response> {
        self.reader
            .get_mut()
            .write_all(format!("{}\r\n", command).as_bytes())?;
        self.parse_response()
    }

    /// Provide user account after login
    /// # Arguments
    /// account string representing the user account for login
    ///
    /// # Errors
    /// Errors when fialing to write to server or to parse response.
    pub fn account(&mut self, account: String) -> Result<()> {
        match self.write_cmd(format!("ACCT {}", account))?.code {
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
    /// ```no_compile
    /// fn main() -> Result<()> {
    ///     let mut client = FtpClient::connect("127.0.0.1:21")?;
    ///     client.login("user", "passowrd")?;
    ///
    ///     let mut source = std::fs::File::open("log.txt").expect("Opening file");
    ///     client.put("/home/will/code/log.txt".into(), &mut source)?;
    ///     client.logout()?;
    ///     Ok(())
    /// }
    ///
    /// ```
    /// # Errors
    /// Errors when fialing to write to server or to parse response or due to connection problems.
    pub fn get(&mut self, file: String, dest: &mut impl Write) -> Result<()> {
        let mut stream = self.pasv()?;
        let response = self.write_cmd(format!("RETR {}", file))?;
        if response.code != FILE_OK {
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
    /// ```no_compile
    ///  use simpleftp::client::FtpClient;
    ///  use simpleftp::Result;
    ///
    /// fn main() -> Result<()> {
    ///     let mut client = FtpClient::connect("127.0.0.1:21")?;
    ///     client.login("user", "passowrd/")?;
    ///
    ///     let mut destination = std::fs::File::create("log.txt").expect("Opening file");
    ///     client.put("/home/will/code/log.txt".into(), &mut destination)?;
    ///     client.logout()?;
    ///     Ok(())
    /// }
    /// ```
    /// # Errors
    /// Errors when fialing to write to server or to parse response or due to connection problems.
    /// May also fail when reading from the source stream.
    pub fn put(&mut self, file: String, source: &mut impl Read) -> Result<()> {
        let mut stream = self.pasv()?;
        let response = self.write_cmd(format!("STOR {}", file))?;

        if response.code != FILE_OK {
            return Err(FtpError::CommandError("Could not process file STOR".into()));
        }
        #[cfg(feature = "debug")]
        println!("Copying file:{}", file);

        std::io::copy(source, &mut stream)?;

        #[cfg(feature = "debug")]
        println!("Closing connection");
        match self.parse_response()?.code {
            CLOSING_DATA_CONNECTION => Ok(()),
            _ => Err(FtpError::ConnectionError("Error closing connection".into())),
        }
    }

    /// Sends a NO OPERATION command
    ///
    /// # Errors
    /// When the connection to server fails or when the server provides invalid response.
    pub fn noop(&mut self) -> Result<()> {
        match self.write_cmd("NOOP".into())?.code {
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
    /// # use simpleftp::client::FtpClient;
    /// let mut client = FtpClient::connect("127.0.0.1:21").unwrap();
    /// client.login("user", "password").unwrap();
    /// client.rename(
    ///     "/home/will/code/log.txt".into(),
    ///     "/home/will/code/text.log".into(),
    /// ).unwrap();
    /// client.logout().unwrap();
    /// ```
    ///
    /// # Errors
    /// Due to connection errors with the server, incorrect filenames or server response.
    pub fn rename(&mut self, from: String, to: String) -> Result<()> {
        let response = self.write_cmd(format!("RNFR {}", from))?;
        if response.code != FILE_ACTION_PENDING {
            return Err(FtpError::CommandError(format!(
                "Could not rename file: {}",
                response.code
            )));
        }

        let response = self.write_cmd(format!("RNTO {}", to))?;
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
    /// # use simpleftp::client::FtpClient;
    /// let mut client = FtpClient::connect("127.0.0.1:21").unwrap();
    /// client.login("user", "password").unwrap();
    /// client.delete("/home/will/code/log.txt".into()).unwrap();
    /// client.logout().unwrap();
    /// ```
    pub fn delete(&mut self, file: String) -> Result<()> {
        let response = self.write_cmd(format!("DELE {}", file))?;
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
        let response = self.write_cmd("PASV".into())?;
        let code = response.code;
        if code != PASSIVE_MODE {
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

    /// Disconnect from the server
    ///
    /// # Errors
    /// On the strange circumstances the server refuses the logout operation
    /// or does not recognize the command.
    pub fn logout(&mut self) -> Result<()> {
        match self.write_cmd("QUIT".into())?.code {
            SERVICE_CLOSING => Ok(()),
            other => Err(FtpError::LoginError(format!("Ivalid response {}", other))),
        }
    }

    /// reads a response and returns the server's response
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
            while !new_line.starts_with(&response[0..4]) {
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
    fn extract_pasv_address(response: &String) -> Result<String> {
        let re = regex::Regex::new(r"(\d{1,3}),(\d{1,3}),(\d{1,3}),(\d{1,3}),(\d{1,3}),(\d{1,3})")
            .unwrap();
        let format_error =
            FtpError::ResponseError(format!("Invalid PASV response from server: {}", response));
        let ipgroup = re.captures(&response).ok_or(format_error.clone())?;
        if ipgroup.len() < 6 {
            return Err(format_error.clone());
        } else {
            let port_lower: u32 = ipgroup[6].parse().unwrap();
            let port_upper: u32 = ipgroup[5].parse().unwrap();

            let full_port = port_upper * 256 + port_lower;
            Ok(format!(
                "{}.{}.{}.{}:{}",
                &ipgroup[1], &ipgroup[2], &ipgroup[3], &ipgroup[4], full_port
            ))
        }
    }
}
