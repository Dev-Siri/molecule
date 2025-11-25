use anyhow::Result;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

use crate::molecule::Molecule;

trait MoleculeTcpHandle {
    async fn handle_client(&self, client: &mut TcpStream) -> Result<()>;
}

pub trait MoleculeTcpApi {
    async fn start_tcp(&self) -> Result<()>;
}

impl MoleculeTcpApi for Molecule {
    async fn start_tcp(&self) -> Result<()> {
        let bind_addr = format!("{}:{}", self.addr, self.port);
        let tcp_listener = TcpListener::bind(&bind_addr).await?;
        let (mut stream, socket) = tcp_listener.accept().await?;

        log::info!("Client connected with IP: {}", socket.ip());

        self.handle_client(&mut stream).await
    }
}

impl MoleculeTcpHandle for Molecule {
    async fn handle_client(&self, client: &mut TcpStream) -> Result<()> {
        let mut buf: Vec<u8> = vec![0u8; 1024];
        let buf_size = client.read(&mut buf).await?;

        println!("sizeof(buf) = {}", buf_size);
        println!("buf in UTF-8: {}", String::from_utf8(buf)?);

        client.write("hello world!".to_string().as_bytes()).await?;

        Ok(())
    }
}
