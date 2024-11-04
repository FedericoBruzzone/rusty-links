#![feature(rustc_private)]

fn main() {
    env_logger::init();
    rustc_ex::instrument::driver_main(rustc_ex::RustcEx);
}
