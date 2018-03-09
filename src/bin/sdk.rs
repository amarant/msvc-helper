extern crate env_logger;
extern crate msvc_helper;

use msvc_helper::windows_sdk::{get_latest_windows_sdk, get_windows_sdk};

fn main() {
    env_logger::init();

    println!("{:?}", get_windows_sdk());
    println!("{:?}", get_latest_windows_sdk());
}
