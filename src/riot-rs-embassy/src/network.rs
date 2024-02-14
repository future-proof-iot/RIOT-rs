use embassy_net::Stack;

use crate::NetworkDevice;

#[allow(dead_code)]
pub const ETHERNET_MTU: usize = 1514;

pub type NetworkStack = Stack<NetworkDevice>;

#[embassy_executor::task]
pub async fn net_task(stack: &'static Stack<NetworkDevice>) -> ! {
    stack.run().await
}

pub fn config() -> embassy_net::Config {
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
