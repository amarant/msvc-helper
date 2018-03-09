extern crate msvc_helper;

use msvc_helper::toolchain::{get_lasted_platform_toolset, get_toolchains};

fn main() {
    println!("{:?}", get_toolchains());
    println!("{:?}", get_lasted_platform_toolset());
}
