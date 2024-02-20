use core::cell::OnceCell;

use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_sync::blocking_mutex::CriticalSectionMutex;

use crate::sendcell::SendCell;
use crate::NetworkDevice;

#[allow(dead_code)]
pub const ETHERNET_MTU: usize = 1514;

pub type NetworkStack = Stack<NetworkDevice>;

pub(crate) static STACK: CriticalSectionMutex<OnceCell<SendCell<&'static NetworkStack>>> =
    CriticalSectionMutex::new(OnceCell::new());

pub async fn network_stack() -> Option<&'static NetworkStack> {
    let spawner = Spawner::for_current_executor().await;
    STACK.lock(|cell| cell.get().map(|x| *x.get(spawner).unwrap()))
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
