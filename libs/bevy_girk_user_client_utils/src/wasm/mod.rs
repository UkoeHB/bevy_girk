//module tree
mod local_player;
mod multiplayer;

//API exports
pub use crate::wasm::local_player::*;
pub use crate::wasm::multiplayer::*;
