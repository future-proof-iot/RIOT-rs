//! Helpers for udp_nal -- conversion and error types

use embedded_nal_async as nal;
use riot_rs_embassy::embassy_net::udp;
use smoltcp::wire::{IpAddress, IpEndpoint};

pub(super) fn sockaddr_nal2smol(sockaddr: nal::SocketAddr) -> Result<IpEndpoint, Error> {
    match sockaddr {
        #[allow(unused)]
        nal::SocketAddr::V4(sockaddr) => {
            #[cfg(feature = "proto-ipv4")]
            return Ok(IpEndpoint {
                addr: smoltcp::wire::Ipv4Address(sockaddr.ip().octets()).into(),
                port: sockaddr.port(),
            });
            #[cfg(not(feature = "proto-ipv4"))]
            return Err(Error::AddressFamilyUnavailable);
        }
        #[allow(unused)]
        nal::SocketAddr::V6(sockaddr) => {
            #[cfg(feature = "proto-ipv6")]
            return Ok(IpEndpoint {
                addr: smoltcp::wire::Ipv6Address(sockaddr.ip().octets()).into(),
                port: sockaddr.port(),
            });
            #[cfg(not(feature = "proto-ipv6"))]
            return Err(Error::AddressFamilyUnavailable);
        }
    }
}

pub(super) fn sockaddr_smol2nal(endpoint: IpEndpoint) -> nal::SocketAddr {
    match endpoint.addr {
        // Let's hope those are in sync; what we'll really need to know is whether smoltcp has the
        // relevant flags set (but we can't query that).
        #[cfg(feature = "proto-ipv4")]
        IpAddress::Ipv4(addr) => {
            embedded_nal_async::SocketAddrV4::new(addr.0.into(), endpoint.port).into()
        }
        #[cfg(feature = "proto-ipv6")]
        IpAddress::Ipv6(addr) => {
            embedded_nal_async::SocketAddrV6::new(addr.0.into(), endpoint.port).into()
        }
    }
}

/// Is the IP address in this type the unspecified address?
///
/// FIXME: What of ::ffff:0.0.0.0? Is that expected to bind to all v4 addresses?
pub(super) fn is_unspec_ip(addr: nal::SocketAddr) -> bool {
    match addr {
        nal::SocketAddr::V4(sockaddr) => sockaddr.ip().octets() == [0; 4],
        nal::SocketAddr::V6(sockaddr) => sockaddr.ip().octets() == [0; 16],
    }
}

/// Unified error type for [embedded_nal_async] operations on UDP sockets
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Error stemming from failure to send
    RecvError(udp::RecvError),
    /// Error stemming from failure to send
    SendError(udp::SendError),
    /// Error stemming from failure to bind to an address/port
    BindError(udp::BindError),
    /// Error stemming from failure to represent the given address family for lack of enabled
    /// embassy-net features
    AddressFamilyUnavailable,
}

impl embedded_io_async::Error for Error {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        match self {
            Self::SendError(udp::SendError::NoRoute) => {
                embedded_io_async::ErrorKind::AddrNotAvailable
            }
            Self::BindError(udp::BindError::NoRoute) => {
                embedded_io_async::ErrorKind::AddrNotAvailable
            }
            Self::AddressFamilyUnavailable => embedded_io_async::ErrorKind::AddrNotAvailable,
            // These should not happen b/c our sockets are typestated.
            Self::SendError(udp::SendError::SocketNotBound) => embedded_io_async::ErrorKind::Other,
            Self::BindError(udp::BindError::InvalidState) => embedded_io_async::ErrorKind::Other,
            // This should not happen b/c in embedded_nal_async this is not expressed through an
            // error.
            // FIXME we're not there in this impl yet.
            Self::RecvError(udp::RecvError::Truncated) => embedded_io_async::ErrorKind::Other,
        }
    }
}
impl From<udp::BindError> for Error {
    fn from(err: udp::BindError) -> Self {
        Self::BindError(err)
    }
}
impl From<udp::RecvError> for Error {
    fn from(err: udp::RecvError) -> Self {
        Self::RecvError(err)
    }
}
impl From<udp::SendError> for Error {
    fn from(err: udp::SendError) -> Self {
        Self::SendError(err)
    }
}
