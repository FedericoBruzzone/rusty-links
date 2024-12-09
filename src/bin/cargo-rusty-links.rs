#![feature(rustc_private)]

fn main() {
    env_logger::init();
    rusty_links::instrument::cli_main(
        rusty_links::RustyLinks,
        rusty_links::RustyLinks::before_exec,
        rusty_links::RustyLinks::after_exec,
    );
}
