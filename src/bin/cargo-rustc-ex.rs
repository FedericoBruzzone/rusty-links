#![feature(rustc_private)]

fn main() {
    env_logger::init();
    rustc_ex::instrument::cli_main(rustc_ex::RustcEx);
}
