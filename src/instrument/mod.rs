// #![feature(rustc_private)]
// extern crate rustc_driver;
// extern crate rustc_interface;
// extern crate rustc_session;

#[doc(hidden)]
pub use cargo_metadata::camino::Utf8Path;
pub use cli::cli_main;
pub use driver::driver_main;
pub use plugin::{CrateFilter, RustcPlugin, RustcPluginArgs};

mod cli;
mod driver;
mod plugin;
