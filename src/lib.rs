#[macro_use]
#[cfg(feature = "glium")]
extern crate glium;

#[macro_use]
extern crate lazy_static;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "server")]
pub mod server;

pub mod blocks;
pub mod packet;
