use simpleftp::FtpClient;
use simpleftp::Result;

fn main() -> Result<()> {
    let mut client = FtpClient::connect("test.rebex.net:21")?;
    client.login("demo", "password")?;

    client.get("/readme.txt", &mut std::fs::File::create("readme.txt")?)?;

    client.logout()?;
    Ok(())
}
