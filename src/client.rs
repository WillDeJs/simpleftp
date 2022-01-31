use super::status::*;
use super::*;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::Ipv4Addr;
use std::net::TcpStream;
use std::net::ToSocketAddrs;

pub struct FtpClient {
    reader: BufReader<TcpStream>,
}

impl FtpClient {
    /// Open a FTP connection
    ///
    /// # Arguments
    /// address     Server address to connect
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
        let reader = BufReader::new(
            TcpStream::connect(address)
                .map_err(|_| FtpError::ConnectionError("Could not oppen connection".into()))?,
        );
        let mut client = FtpClient { reader };

        if client.parse_response()? != SERVICE_READY {
            return Err(FtpError::ConnectionError(
                "Server not ready for conenctions".into(),
            ));
        }
        Ok(client)
    }

    /// Perform Login to server.
    /// # Arguments
    /// username    username for login
    /// password    password for given us er
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
        let response = self.write_cmd(format!("USER {}\r\n", username))?;

        if response != NEED_PASSWORD && response != LOGGED_IN {
            return Err(FtpError::LoginError(format!(
                "Could not authenticate: {}",
                response
            )));
        }

        // send password
        let response = self.write_cmd(format!("PASS {}\r\n", password))?;
        if response != LOGGED_IN {
            return Err(FtpError::LoginError(format!(
                "Invalid username/password combination: {}",
                response
            )));
        }

        Ok(())
    }

    /// Write a command to the server
    /// # Arguments
    /// command     text command to be sent to server
    /// # Errors
    /// Errors when faiing to write to server or to parse a response.
    pub fn write_cmd(&mut self, command: String) -> Result<usize> {
        self.reader
            .get_mut()
            .write_all(command.as_bytes())
            .map_err(|_| FtpError::LoginError("Could not write to server".into()))?;
        self.parse_response()
    }

    pub fn account(&mut self, account: String) -> Result<()> {
        match self.write_cmd(format!("ACCT {}\r\n", account)) {
            Ok(LOGGED_IN) => Ok(()),
            Ok(x) => Err(FtpError::LoginError("Invalid account information".into())),
            Err(e) => Err(e),
        }
    }
    pub fn get(&mut self, file: String) -> Result<BufReader<TcpStream>> {
        let stream = self.pasv()?;
        let reader = BufReader::new(stream);
        let response = self.write_cmd(format!("RETR {}\r\n", file))?;
        if response != FILE_OK {
            return Err(FtpError::CommandError(
                "Could not process file retrieve".into(),
            ));
        }
        Ok(reader)
    }
    fn pasv(&mut self) -> Result<TcpStream> {
        let mut response = String::new();
        self.reader
            .get_mut()
            .write_all("PASV\r\n".as_bytes())
            .map_err(|_| FtpError::ResponseError("Could not read server response".into()))?;

        self.reader
            .read_line(&mut response)
            .map_err(|_| FtpError::ConnectionError("Could not read data from server".into()))?;

        #[cfg(feature = "debug")]
        print!("Parsing: {}", response);

        let code: usize = response[0..3].parse().map_err(|_| {
            FtpError::ResponseError(format!(
                "Invalid response code form server: {}",
                &response.clone()
            ))
        })?;
        if code != PASSIVE_MODE {
            return Err(FtpError::ResponseError(format!(
                "Invalid response code from server: {}",
                response
            )));
        }
        let address = Self::extract_pasv_address(&response)?;

        #[cfg(feature = "debug")]
        println!("{}", address);

        let connection = TcpStream::connect(address).map_err(|_| {
            FtpError::ConnectionError(format!("Could not open data connection: {}", response))
        })?;
        Ok(connection)
    }

    /// reads a response and returns the mesage code
    fn parse_response(&mut self) -> Result<usize> {
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
                self.reader.read_line(&mut new_line).map_err(|_| {
                    FtpError::ResponseError("Could not read server response".into())
                })?;
            }
        }

        Ok(code)
    }

    fn extract_pasv_address(response: &String) -> Result<String> {
        let re = regex::Regex::new(r"(\d{1,3}),(\d{1,3}),(\d{1,3}),(\d{1,3}),(\d{1,3}),(\d{1,3})")
            .unwrap();
        let format_error =
            FtpError::ResponseError(format!("Invalid PASV response from server: {}", response));
        let ipgroup = re.captures(&response).ok_or(format_error.clone())?;
        if ipgroup.len() < 6 {
            return Err(format_error.clone());
        } else {
            let port_lower: u32 = ipgroup[6].parse().map_err(|_| format_error.clone())?;
            let port_upper: u32 = ipgroup[5].parse().map_err(|_| format_error.clone())?;

            let full_port = port_upper * 256 + port_lower;
            Ok(format!(
                "{}.{}.{}.{}:{}",
                &ipgroup[1], &ipgroup[2], &ipgroup[3], &ipgroup[4], full_port
            ))
        }
    }
}
