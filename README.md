# simpleftp
Naive toy FTP Client  </br>
Non-encrypted only implement basic functionality.

## Example
``` rs
use simpleftp::FtpClient;
use simpleftp::Result;

fn main() -> Result<()> {
    // connect to server
    let mut client = FtpClient::connect("test.rebex.net:21")?;
    client.login("demo", "password")?;

    // download file
    let mut readme = std::fs::File::create("readme.txt")?;
    client.get("/readme.txt", &mut readme)?;

    // disconnect from server
    client.logout()?;
    Ok(())
}

```
## Supported:
- [x] USER
- [x] PASS
- [x] ACCT
- [x] CWD
- [x] CDUP
- [x] SMNT
- [x] QUIT
- [ ] ~REIN~
- [ ] ~PORT~ 
- [x] PASV
- [ ] TYPE
- [ ] STRU
- [ ] MODE
- [x] RETR
- [x] STOR
- [x] STOU
- [x] APPE
- [x] ALLO
- [ ] ~REST~
- [x] RNFR
- [x] RNTO
- [x] ABOR
- [x] DELE
- [x] RMD
- [x] MKD
- [x] PWD
- [x] LIST
- [x] NLST
- [ ] ~SITE~
- [x] SYST
- [x] STAT
- [x] HELP
- [x] NOOP
