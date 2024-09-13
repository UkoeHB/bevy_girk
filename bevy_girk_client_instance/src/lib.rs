#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//module tree
mod client_factory;
mod client_instance_command;
mod client_instance_plugin;
mod client_instance_report;
mod local_game_manager;

//API exports
pub use client_factory::*;
pub use client_instance_command::*;
pub use client_instance_plugin::*;
pub use client_instance_report::*;
pub use local_game_manager::*;
