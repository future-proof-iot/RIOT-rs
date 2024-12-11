//! UDP sockets usable through [`embedded_nal_async`]
//!
//! The full [`embedded_nal_async::UdpStack`] is *not* implemented at the moment: As its API allows
//! arbitrary creation of movable sockets, embassy's [`udp::UdpSocket`] type could only be crated if
//! the NAL stack had a pre-allocated pool of sockets with their respective buffers. Nothing rules
//! out such a type, but at the moment, only the bound or connected socket types are implemented
//! with their own constructors from an embassy [`crate::Stack`] -- for many applications, those are
//! useful enough. (FIXME: Given we construct from Socket, Stack could really be implemented on
//! `Cell<Option<Socket>>` by `.take()`ing, couldn't it?)
//!
//! The constructors of the various socket types mimic the [`UdpStack`]'s socket creation functions,
//! but take an owned (uninitialized) Socket instead of a shared stack.
//!
//! No `bind_single` style constructor is currently provided. FIXME: Not sure we have all the
//! information at bind time to specialize a wildcard address into a concrete address and return
//! it. Should the NAL trait be updated to disallow using wildcard addresses on `bind_single`, and
//! merely allow unspecified ports to get an ephemeral one?

use core::future::poll_fn;
use core::net::SocketAddr;

use embassy_net::{udp, IpAddress, IpEndpoint};
use embedded_nal_async as nal;

mod util;
pub use util::Error;
use util::{is_unspec_ip, sockaddr_nal2smol, sockaddr_smol2nal};

#[expect(
    dead_code,
    reason = "pub item is being prepared for embedded-nal-async where it will be reachable publicly"
)]
pub struct ConnectedUdp<'a> {
    remote: IpEndpoint,
    // The local port is stored in the socket, as it gets bound. This value is populated lazily:
    // embassy only decides at udp::Socket::dispatch time whence to send, and while we could
    // duplicate the code for the None case of the local_address by calling the right
    // get_source_address function, we'd still need an interface::Context / an interface to call
    // this through, and AFAICT we don't get access to that.
    local: Option<IpAddress>,
    socket: udp::UdpSocket<'a>,
}

/// A UDP socket that has been bound locally (either to a unique address or just to a port)
///
/// Its operations are accessible through the [`nal::UnconnectedUdp`] trait.
pub struct UnconnectedUdp<'a> {
    socket: udp::UdpSocket<'a>,
}

#[allow(
    dead_code,
    clippy::unused_async,
    clippy::missing_errors_doc,
    reason = "pub item is being prepared for embedded-nal-async where it will be reachable publicly"
)]
impl<'a> ConnectedUdp<'a> {
    /// Create a [`ConnectedUdp`] by assigning it a remote and a concrete local address
    ///
    /// ## Prerequisites
    ///
    /// The `socket` must be open (in the sense of smoltcp's `.is_open()`) -- unbound and
    /// unconnected.
    pub async fn connect_from(
        mut socket: udp::UdpSocket<'a>,
        local: SocketAddr,
        remote: SocketAddr,
    ) -> Result<Self, Error> {
        socket.bind(sockaddr_nal2smol(local)?)?;

        Ok(ConnectedUdp {
            remote: sockaddr_nal2smol(remote)?,
            // FIXME: We could check if local was fully (or sufficiently, picking the port from the
            // socket) specified and store if yes -- for a first iteration, leaving that to the
            // fallback path we need anyway in case local is [::].
            local: None,
            socket,
        })
    }

    /// Create a [`ConnectedUdp`] by assigning it a remote and a local address (the latter may
    /// happen lazily)
    ///
    /// ## Prerequisites
    ///
    /// The `socket` must be open (in the sense of smoltcp's `.is_open()`) -- unbound and
    /// unconnected.
    pub async fn connect(socket: udp::UdpSocket<'a>, /*, ... */) -> Result<Self, udp::BindError> {
        // This is really just a copy of the provided `embedded_nal::udp::UdpStack::connect` method
        todo!("use {:p}", &socket)
    }
}

#[allow(
    dead_code,
    clippy::unused_async,
    clippy::missing_errors_doc,
    reason = "pub item is being prepared for embedded-nal-async where it will be reachable publicly"
)]
impl<'a> UnconnectedUdp<'a> {
    /// Create an [`UnconnectedUdp`].
    ///
    /// The `local` address may be anything from fully specified (address and port) to fully
    /// unspecified (port 0, all-zeros address).
    ///
    /// ## Prerequisites
    ///
    /// The `socket` must be open (in the sense of smoltcp's `.is_open()`) -- unbound and
    /// unconnected.
    pub async fn bind_multiple(
        mut socket: udp::UdpSocket<'a>,
        local: SocketAddr,
    ) -> Result<Self, Error> {
        socket.bind(sockaddr_nal2smol(local)?)?;

        Ok(UnconnectedUdp { socket })
    }
}

impl nal::UnconnectedUdp for UnconnectedUdp<'_> {
    type Error = Error;
    async fn send(
        &mut self,
        local: SocketAddr,
        remote: SocketAddr,
        buf: &[u8],
    ) -> Result<(), Error> {
        // While the underlying layers probably don't care, we're not passing on the port
        // information, so the underlying layers won't even have a *chance* to care if we don't
        // check here.
        debug_assert!(
            local.port() == 0 || local.port() == self.socket.with(|s, _| s.endpoint().port),
            "Port of local address, when given, must match bound port."
        );

        let remote_endpoint = udp::UdpMetadata {
            local_address: if is_unspec_ip(local) {
                None
            } else {
                // A conversion of the addr part only might be cheaper, but would also mean we need
                // two functions
                Some(sockaddr_nal2smol(local)?.addr)
            },
            ..sockaddr_nal2smol(remote)?.into()
        };
        poll_fn(move |cx| self.socket.poll_send_to(buf, remote_endpoint, cx)).await?;
        Ok(())
    }
    async fn receive_into(
        &mut self,
        buf: &mut [u8],
    ) -> Result<(usize, SocketAddr, SocketAddr), Error> {
        // FIXME: The truncation is an issue -- we may need to change poll_recv_from to poll_recv
        // and copy from the slice ourselves to get the trait's behavior
        let (size, metadata) = poll_fn(|cx| self.socket.poll_recv_from(buf, cx)).await?;
        Ok((
            size,
            sockaddr_smol2nal(IpEndpoint {
                addr: metadata
                    .local_address
                    .expect("Local address is always populated on receive"),
                port: self.socket.with(|s, _| s.endpoint().port),
            }),
            sockaddr_smol2nal(metadata.endpoint),
        ))
    }
}
