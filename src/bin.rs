extern crate env_logger;
extern crate msvc_helper;

use msvc_helper::get_windows_sdk;

fn main() {
    env_logger::init();

    let windows_sdk = get_windows_sdk();
    println!("{:?}", windows_sdk);
}
