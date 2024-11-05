#![feature(rustc_private)]

fn main() {
    env_logger::init();
    rusty_links::instrument::driver_main(rusty_links::RustyLinks);
}
