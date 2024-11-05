#![feature(rustc_private)]

fn main() {
    env_logger::init();
    rusty_links::instrument::cli_main(rusty_links::RustyLinks);
}
