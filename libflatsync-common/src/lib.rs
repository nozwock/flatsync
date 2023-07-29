#[rustfmt::skip]
pub mod config {
    #![allow(dead_code)]

    include!(concat!(env!("CODEGEN_BUILD_DIR"), "/config.rs"));
}

mod error;

pub use error::Error;

pub mod dbus;
pub mod models;
pub use models::*;
