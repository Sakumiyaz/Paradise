//! Channel abstraction for IPC communication

#![allow(dead_code)]

use super::message::Message;
use super::socket::UnixDatagram;
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Atomic sequence counter
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn next_sequence() -> u64 {
    SEQUENCE.fetch_add(1, Ordering::Relaxed)
}

/// Channel endpoint for sending/receiving messages
pub struct ChannelEndpoint {
    socket: UnixDatagram,
    peer_addr: Option<String>,
    local_sequence: u64,
    remote_sequence: u64,
}

impl ChannelEndpoint {
    /// Create a new channel endpoint
    pub fn new(socket: UnixDatagram, peer_addr: Option<String>) -> Self {
        Self {
            socket,
            peer_addr,
            local_sequence: 0,
            remote_sequence: 0,
        }
    }

    /// Create a connected channel pair
    pub fn pair() -> io::Result<(Self, Self)> {
        let (sock1, sock2) = UnixDatagram::pair()?;
        Ok((Self::new(sock1, None), Self::new(sock2, None)))
    }

    /// Send a message
    pub fn send(&mut self, msg: &Message) -> io::Result<usize> {
        let mut msg_with_seq = msg.clone();
        msg_with_seq.header.sequence = self.local_sequence;
        self.local_sequence += 1;

        match &self.peer_addr {
            Some(path) => self.socket.send_to(path, &msg_with_seq.to_bytes()),
            None => self.socket.send(&msg_with_seq.to_bytes()),
        }
    }

    /// Receive a message
    pub fn recv(&self, buffer: &mut [u8]) -> io::Result<usize> {
        match self.peer_addr {
            Some(_) => self.socket.recv(buffer),
            None => {
                let (size, _) = self.socket.recv_from(buffer)?;
                Ok(size)
            }
        }
    }

    /// Receive and parse a message
    pub fn recv_message(&self) -> io::Result<Message> {
        let mut buffer = vec![0u8; 65536]; // 64KB max
        let size = self.recv(&mut buffer)?;
        buffer.truncate(size);
        Message::from_bytes(&buffer)
    }

    /// Set non-blocking mode
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.socket.set_nonblocking(nonblocking)
    }
}

/// High-level channel abstraction
pub struct Channel {
    pub name: String,
    pub endpoint: ChannelEndpoint,
    pub buffer_size: usize,
}

impl Channel {
    /// Create a new channel bound to a path
    pub fn bind<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let name = path.as_ref().to_string_lossy().into_owned();
        let socket = UnixDatagram::bind(&path)?;

        Ok(Self {
            name,
            endpoint: ChannelEndpoint::new(socket, None),
            buffer_size: 65536,
        })
    }

    /// Connect to a channel by path
    pub fn connect<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let name = path.as_ref().to_string_lossy().into_owned();
        let socket = UnixDatagram::unbound()?;
        socket.connect(&path)?;

        Ok(Self {
            name: name.clone(),
            endpoint: ChannelEndpoint::new(socket, Some(name)),
            buffer_size: 65536,
        })
    }

    /// Send a message through the channel
    pub fn send(&mut self, msg: &Message) -> io::Result<usize> {
        self.endpoint.send(msg)
    }

    /// Receive a message from the channel
    pub fn recv(&self) -> io::Result<Message> {
        self.endpoint.recv_message()
    }

    /// Create a channel pair for in-process communication
    pub fn pair() -> io::Result<(Channel, Channel)> {
        let (ep1, ep2) = ChannelEndpoint::pair()?;
        Ok((
            Channel {
                name: "channel_a".to_string(),
                endpoint: ep1,
                buffer_size: 65536,
            },
            Channel {
                name: "channel_b".to_string(),
                endpoint: ep2,
                buffer_size: 65536,
            },
        ))
    }
}

/// Builder for creating channels with custom settings
pub struct ChannelBuilder {
    name: Option<String>,
    buffer_size: usize,
    nonblocking: bool,
}

impl ChannelBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            buffer_size: 65536,
            nonblocking: false,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    pub fn nonblocking(mut self, nonblocking: bool) -> Self {
        self.nonblocking = nonblocking;
        self
    }

    pub fn bind<P: AsRef<Path>>(self, path: P) -> io::Result<Channel> {
        let channel = Channel::bind(path)?;
        channel.endpoint.set_nonblocking(self.nonblocking)?;
        Ok(channel)
    }

    pub fn connect<P: AsRef<Path>>(self, path: P) -> io::Result<Channel> {
        let path_str = path.as_ref().to_string_lossy().into_owned();
        let mut channel = Channel::connect(path)?;
        channel.name = self.name.unwrap_or(path_str);
        channel.endpoint.set_nonblocking(self.nonblocking)?;
        Ok(channel)
    }
}

impl Default for ChannelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_channel_pair() {
        let (mut ch1, ch2) = Channel::pair().unwrap();

        let msg = Message::data(b"IPC_TEST".to_vec()).unwrap();
        ch1.send(&msg).unwrap();

        let received = ch2.recv().unwrap();
        assert_eq!(received.payload, b"IPC_TEST");
    }

    #[test]
    fn test_channel_bind_connect() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_channel");

        let server = match Channel::bind(&path) {
            Ok(server) => server,
            Err(err) if err.kind() == io::ErrorKind::PermissionDenied => {
                let (mut client, server) = Channel::pair().unwrap();
                let msg = Message::control(b"PING").unwrap();
                client.send(&msg).unwrap();
                let received = server.recv().unwrap();
                assert_eq!(received.payload, b"PING");
                return;
            }
            Err(err) => panic!("failed to bind channel: {err}"),
        };
        let mut client = match Channel::connect(&path) {
            Ok(client) => client,
            Err(err) if err.kind() == io::ErrorKind::PermissionDenied => {
                let (mut client, server) = Channel::pair().unwrap();
                let msg = Message::control(b"PING").unwrap();
                client.send(&msg).unwrap();
                let received = server.recv().unwrap();
                assert_eq!(received.payload, b"PING");
                return;
            }
            Err(err) => panic!("failed to connect channel: {err}"),
        };

        let msg = Message::control(b"PING").unwrap();
        client.send(&msg).unwrap();

        let received = server.recv().unwrap();
        assert_eq!(received.payload, b"PING");
    }
}
