#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod packet_handling;

#[cfg(feature = "transport")]
mod server_setup;
#[cfg(feature = "transport")]
mod renet_setup;
#[cfg(feature = "test")]
mod test_network;

pub use packet_handling::*;

#[cfg(feature = "transport")]
pub use server_setup::*;
#[cfg(feature = "transport")]
pub use renet_setup::*;
#[cfg(feature = "test")]
pub use test_network::*;
