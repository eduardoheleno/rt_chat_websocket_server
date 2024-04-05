use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8081").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("{:?}", e);
                        return;
                    }
                };

                println!("{n}");

                if let Err(e) = socket.write_all("test".as_bytes()).await {
                    eprintln!("{:?}", e);
                    return;
                }
            }
        });
    }
}
