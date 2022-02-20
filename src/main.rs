use simpleftp::FtpClient;
use simpleftp::Result;

fn main() -> Result<()> {
    let mut client = FtpClient::connect("test.rebex.net:21")?;
    client.login("demo", "password")?;

    let mut readme = std::fs::File::create("readme.txt")?;
    client.get("/readme.txt", &mut readme)?;

    client.logout()?;
    Ok(())
}
