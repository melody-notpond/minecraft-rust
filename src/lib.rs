#[macro_use]
#[cfg(feature = "glium")]
extern crate glium;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "server")]
pub mod server;

pub mod blocks;
pub mod packet;
