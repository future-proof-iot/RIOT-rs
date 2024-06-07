//! To provide a custom network configuration, use the `riot_rs::config` attribute macro.

use embassy_net::Stack;
use embassy_sync::once_lock::OnceLock;

use crate::sendcell::SendCell;
use crate::NetworkDevice;

#[allow(dead_code)]
pub const ETHERNET_MTU: usize = 1514;

pub type NetworkStack = Stack<NetworkDevice>;

pub(crate) static STACK: OnceLock<SendCell<&'static NetworkStack>> = OnceLock::new();

pub async fn network_stack() -> Option<&'static NetworkStack> {
    STACK.get().await.get_async().await.copied()
}

#[embassy_executor::task]
pub(crate) async fn net_task(stack: &'static Stack<NetworkDevice>) -> ! {
    stack.run().await
}

pub(crate) fn config() -> embassy_net::Config {
    #[cfg(not(feature = "override-network-config"))]
    {
        embassy_net::Config::dhcpv4(Default::default())
    }
    #[cfg(feature = "override-network-config")]
    {
        extern "Rust" {
            fn riot_rs_network_config() -> embassy_net::Config;
        }
        unsafe { riot_rs_network_config() }
    }
}
