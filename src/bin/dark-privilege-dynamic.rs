// This is an EXE that will dynamically load the DLL at run time.

/*
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA, FreeLibrary};
use windows::Win32::Foundation::HINSTANCE;
use std::ffi::CString;
use std::mem;

// Function signatures for dynamically loaded functions
type WinverFn = unsafe extern "C" fn();
type SysinfoFn = unsafe extern "C" fn();
*/

use env_logger::Env;
//use log::{debug, error, log_enabled, info, Level};
use log::{info};

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("dynamic exe");
    /*
    // Load the DLL
    let dll_path = CString::new("dark_privilege_lib.dll").expect("CString::new failed");
    let h_module: HINSTANCE = unsafe { LoadLibraryA(dll_path.as_c_str()) };

    if h_module.is_invalid() {
        panic!("Failed to load DLL");
    }

    // Load the 'exported_winver' function
    let winver_name = CString::new("exported_winver").expect("CString::new failed");
    let winver_ptr = unsafe { GetProcAddress(h_module, winver_name.as_c_str()) };
    if winver_ptr.is_none() {
        panic!("Failed to get winver function");
    }
    let winver: WinverFn = unsafe { mem::transmute(winver_ptr) };

    // Load the 'exported_sysinfo' function
    let sysinfo_name = CString::new("exported_sysinfo").expect("CString::new failed");
    let sysinfo_ptr = unsafe { GetProcAddress(h_module, sysinfo_name.as_c_str()) };
    if sysinfo_ptr.is_none() {
        panic!("Failed to get sysinfo function");
    }
    let sysinfo: SysinfoFn = unsafe { mem::transmute(sysinfo_ptr) };

    // Call the dynamically loaded functions
    unsafe {
        winver();
        sysinfo();
    }

    // Free the DLL
    unsafe {
        FreeLibrary(h_module);
    }
    */
}

