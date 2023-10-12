use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {
    // Bind the listener to the address
    let socket = TcpStream::connect("127.0.0.1:6142").await?;
    let (mut rd, mut wr) = io::split(socket);

    // ceate async task, to write data
    tokio::spawn(async move{
        wr.write_all(b"hello\n").await?;
        wr.write_all(b"world\n").await?;

        // somtetimes, we need to flush the buffer
        Ok::<(), io::Error>(())
    });

    let mut buf = vec![0; 128];

    loop {
        let n = rd.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        println!("GOT {:?}", &buf[..n]);
    }

    Ok(())

}