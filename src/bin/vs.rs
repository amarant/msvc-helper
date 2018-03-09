extern crate msvc_helper;
extern crate winapi;

use msvc_helper::setup_config::SetupConfiguration;

fn main() {
    let config = SetupConfiguration::new().unwrap();
    let iter = config.enum_all_instances().unwrap();
    for instance in iter {
        let instance = instance.unwrap();
        println!("{}", instance);
    }
}
