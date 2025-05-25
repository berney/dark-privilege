// This is an exe that will load the DLL at load time via the import table

use env_logger::Env;
//use log::{debug, error, log_enabled, info, Level};
use log::{info};


/*
// Link the DLL at compile time, so it is loaded automatically at program start
extern "C" {
    fn exported_winver();
    fn exported_sysinfo();
}
*/

#[link(name = "dark_privilege_lib.dll")]
unsafe extern "C" {
    fn hello_world();
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("import exe");

    // Use the DLL functions via the import table
    unsafe {
        hello_world();
        //exported_winver();
        //exported_sysinfo();
    }
}
