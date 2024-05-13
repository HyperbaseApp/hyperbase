use std::{net::SocketAddr, time::Duration};

use anyhow::{Error, Result};
use tokio::{io::AsyncWriteExt, net::TcpStream, time::timeout};

use crate::message::Message;

pub async fn send(address: &SocketAddr, message: Message) -> Result<usize> {
    match message.to_vec() {
        Ok(bytes) => match timeout(Duration::from_secs(5), TcpStream::connect(address)).await {
            Ok(tcp_stream) => {
                let written = tcp_stream?.write(&bytes).await?;
                Ok(written)
            }
            Err(err) => Err(Error::from(err)),
        },
        Err(err) => Err(Error::from(err)),
    }
}
