#[macro_use]
#[cfg(feature = "glium")]
extern crate glium;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "server")]
pub mod server;

pub mod packet;

pub const CHUNK_SIZE: usize = 16;
