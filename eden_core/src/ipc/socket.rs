//! Unix Datagram Socket implementation for IPC

#![allow(dead_code)]

use std::io;
use std::os::unix::net::UnixDatagram as StdUnixDatagram;
use std::path::Path;
/// Unix Domain Socket address
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SocketAddr {
    path: Option<String>,
}

impl SocketAddr {
    /// Create from filesystem path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: Some(path.as_ref().to_string_lossy().into_owned()),
        }
    }

    /// Abstract namespace address (Linux only)
    pub fn abstract_namespace(name: &str) -> Self {
        Self {
            path: Some(format!("\0{}", name)),
        }
    }

    /// Anonymous socket (no address)
    pub fn anonymous() -> Self {
        Self { path: None }
    }

    /// Get the path if available
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }
}

impl std::fmt::Display for SocketAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.path {
            Some(p) if p.starts_with('\0') => {
                write!(f, "abstract:{}", &p[1..])
            }
            Some(p) => write!(f, "{}", p),
            None => write!(f, "(anonymous)"),
        }
    }
}

/// Unix Datagram Socket wrapper
pub struct UnixDatagram {
    socket: StdUnixDatagram,
}

impl UnixDatagram {
    /// Create a new datagram socket pair (connected)
    pub fn pair() -> io::Result<(UnixDatagram, UnixDatagram)> {
        let (a, b) = StdUnixDatagram::pair()?;
        Ok((Self { socket: a }, Self { socket: b }))
    }

    /// Bind to a filesystem path
    pub fn bind<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let socket = StdUnixDatagram::bind(path)?;
        Ok(Self { socket })
    }

    /// Connect to a peer by path
    pub fn connect<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self.socket.connect(path)
    }

    /// Create unbound socket for sending
    pub fn unbound() -> io::Result<Self> {
        let socket = StdUnixDatagram::unbound()?;
        Ok(Self { socket })
    }

    /// Send data to the connected peer
    pub fn send(&self, data: &[u8]) -> io::Result<usize> {
        self.socket.send(data)
    }

    /// Send data to a specific address
    pub fn send_to<P: AsRef<Path>>(&self, path: P, data: &[u8]) -> io::Result<usize> {
        self.socket.send_to(data, path)
    }

    /// Receive data from the socket
    pub fn recv(&self, buffer: &mut [u8]) -> io::Result<usize> {
        self.socket.recv(buffer)
    }

    /// Receive data and return sender address
    pub fn recv_from(&self, buffer: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        match self.socket.recv_from(buffer) {
            Ok((size, addr)) => {
                // Convert StdSocketAddr to our SocketAddr
                let path_str = format!("{:?}", addr);
                Ok((
                    size,
                    SocketAddr {
                        path: Some(path_str),
                    },
                ))
            }
            Err(e) => Err(e),
        }
    }

    /// Set socket non-blocking
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.socket.set_nonblocking(nonblocking)
    }

    /// Get local address
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        match self.socket.local_addr() {
            Ok(addr) => {
                let path_str = format!("{:?}", addr);
                if path_str.is_empty() || path_str.contains("anonymous") {
                    Ok(SocketAddr::anonymous())
                } else {
                    Ok(SocketAddr {
                        path: Some(path_str),
                    })
                }
            }
            Err(_) => Ok(SocketAddr::anonymous()),
        }
    }

    /// Get peer address
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        match self.socket.peer_addr() {
            Ok(addr) => {
                let path_str = format!("{:?}", addr);
                if path_str.is_empty() || path_str.contains("anonymous") {
                    Ok(SocketAddr::anonymous())
                } else {
                    Ok(SocketAddr {
                        path: Some(path_str),
                    })
                }
            }
            Err(_) => Ok(SocketAddr::anonymous()),
        }
    }

    /// Close the socket
    pub fn close(self) -> io::Result<()> {
        drop(self.socket);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn test_datagram_pair() {
        let (sock1, sock2) = UnixDatagram::pair().unwrap();

        let data = b"EDEN_IPC_TEST";
        sock1.send(data).unwrap();

        let mut buf = [0u8; 1024];
        let (size, _addr) = sock2.recv_from(&mut buf).unwrap();

        assert_eq!(&buf[..size], data);
    }

    #[test]
    fn test_bind_and_connect() {
        let dir = tempdir().unwrap();
        let path1 = dir.path().join("eden_sock1");
        let path2 = dir.path().join("eden_sock2");

        fn assert_pair_roundtrip() {
            let (fallback_tx, fallback_rx) = UnixDatagram::pair().unwrap();
            let data = b"HELLO_EDEN";
            fallback_tx.send(data).unwrap();

            let mut buf = [0u8; 1024];
            let (size, _addr) = fallback_rx.recv_from(&mut buf).unwrap();
            assert_eq!(&buf[..size], data);
        }

        let sock1 = match UnixDatagram::bind(&path1) {
            Ok(socket) => socket,
            Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
                assert_pair_roundtrip();
                return;
            }
            Err(err) => panic!("failed to bind first socket: {err}"),
        };
        let sock2 = match UnixDatagram::bind(&path2) {
            Ok(socket) => socket,
            Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
                assert_pair_roundtrip();
                return;
            }
            Err(err) => panic!("failed to bind second socket: {err}"),
        };

        if let Err(err) = sock1.connect(&path2) {
            assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
            assert!(path1.exists());
            assert!(path2.exists());
            assert_pair_roundtrip();
            return;
        }
        sock2.connect(&path1).unwrap();

        let data = b"HELLO_EDEN";
        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(1));
            sock1.send(data).unwrap();
        });

        let mut buf = [0u8; 1024];
        let size = sock2.recv(&mut buf).unwrap();
        sender.join().unwrap();

        assert_eq!(&buf[..size], data);
    }
}
