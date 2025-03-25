// This is shared code common to both the .exe and .dll

//use std::default::Default;
//use std::path::PathBuf;
use std::mem::MaybeUninit;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::channel;

use log::{debug, error, info};
use serde::Serialize;
use colored_json::to_colored_json_auto;

use windows::Wdk::System::SystemServices::RtlGetVersion;
use windows::Win32::System::SystemInformation::OSVERSIONINFOW;
/*
use windows::{
    Win32::Foundation::HANDLE,
    Win32::Security::*,
};
*/
use windows::{
    core::*, Win32::Foundation::*, Win32::Security::*, Win32::System::Memory::*,
    Win32::System::Threading::*,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos,
    GetWindowRect,
    GetWindowTextLengthW,
    GetWindowTextW,
    //SetForegroundWindow,
    WindowFromPoint,
};

// RtlGetVersion should be able to pass either OSVERSIONINFOW or OSVERSIONINFOEXW
// https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-rtlgetversion
// But the windows create only allows OSVERSIONINFOW.
//use windows::Win32::System::SystemInformation::OSVERSIONINFOEXW;

use windows_version::{OsVersion, is_server};

use bytesize::ByteSize;

use sysinfo::{
    //Components,
    Disks, Networks, System, Users, Groups,
};

use enigo::{
    Button,
    //Direction::{Click, Press, Release},
    Direction::Click,
    Enigo, Mouse,
    //Key,
    Keyboard, Settings,
};

// XXX TODO This doesn't work
// I'm worried that functions in `src/dll.rs` will end up in static exe (`src/main.rs`)
// Conditionally include the `dll.rs` module only when building the DLL (cdylib)
//#[cfg(any(windows_dll))]
mod dll;

fn get_window_under_cursor() -> Option<HWND> {
    // Create a POINT struct to hold the cursor's position
    let mut cursor_pos = POINT { x: 0, y: 0 };

    // Get the cursor's current position
    unsafe {
        if GetCursorPos(&mut cursor_pos).is_ok() {
            debug!("cursor_pos: ({}, {})", cursor_pos.x, cursor_pos.y);
            // Find the window under the cursor
            let hwnd: HWND = WindowFromPoint(cursor_pos);

            //if !hwnd.0.is_null() {
            if !hwnd.is_invalid() {
                return Some(hwnd);
            }
        }
    }

    None
}


fn get_window_title(hwnd: HWND) -> Option<String> {
    // Get the length of the window's title
    let length = unsafe { GetWindowTextLengthW(hwnd) };
    if length == 0 {
        return None;
    }

    // Create a buffer to hold the window title
    let mut buffer: Vec<u16> = vec![0; (length + 1) as usize];

    // Get the window title text
    unsafe {
        if GetWindowTextW(hwnd, &mut buffer) > 0 {
            // Convert the buffer to a Rust String
            return Some(String::from_utf16_lossy(&buffer[..length as usize]));
        }
    }

    None
}


pub fn paste(message : &String) {
    if let Some(hwnd) = get_window_under_cursor() {
        if let Some(title) = get_window_title(hwnd) {
            info!("Window under cursor: HWND = {:?}, Title = {}", hwnd.0, title);
        } else {
            info!("Window under cursor: HWND = {:?}", hwnd.0);
        }
        // Get the window's rectangle (position and size)
        let mut rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        unsafe {
            if GetWindowRect(hwnd, &mut rect).is_ok() {
                debug!("RECT: (left: {}, top: {}, right: {}, bottom: {})", rect.left, rect.top, rect.right, rect.bottom);
            } else {
                debug!("GetWindowRect failed!");
            }
            // This doesn't work well.
            // For HTML5 Citrix in Edge, the Edge window will become foreground but the cursor
            // in notepad etc inside citrix won't be active, so can't type text.
            //SetForegroundWindow(hwnd)
            //    .expect("SetForegroundWindow failed!");
            let mut enigo = Enigo::new(&Settings::default()).unwrap();
            //thread::sleep(Duration::from_secs(2));
            debug!("screen dimensions: {:?}", enigo.main_display().unwrap());
            debug!("enigo mouse location: {:?}", enigo.location().unwrap());
            // This works with notepad inside HTML5 Citrix in Edge
            enigo.button(Button::Left, Click).unwrap();
            // write text
            enigo
                .text(message)
                .unwrap();
        }
    } else {
        println!("No window found under cursor.");
    }
}


// Privileges can be active or inactive
// This just lists the privileges but doesn't list if they are active.
fn get_privileges() -> Result<()> {
    unsafe {
        let mut token = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)?;

        let mut bytes_required = 0;
        _ = GetTokenInformation(token, TokenPrivileges, None, 0, &mut bytes_required);

        let buffer = Owned::new(LocalAlloc(LPTR, bytes_required as usize)?);

        GetTokenInformation(
            token,
            TokenPrivileges,
            Some(buffer.0 as *mut _),
            bytes_required,
            &mut bytes_required,
        )?;

        let header = &*(buffer.0 as *const TOKEN_PRIVILEGES);

        let privileges =
            std::slice::from_raw_parts(header.Privileges.as_ptr(), header.PrivilegeCount as usize);

        for privilege in privileges {
            let mut name_len = 0;
            _ = LookupPrivilegeNameW(None, &privilege.Luid, PWSTR::null(), &mut name_len);

            let mut name = vec![0u16; (name_len + 1) as usize];
            let name = PWSTR(name.as_mut_ptr());
            LookupPrivilegeNameW(None, &privilege.Luid, name, &mut name_len)?;

            info!("{}", name.display())
        }

        Ok(())
    }
}

/*
fn get_privileges() -> Result<()> {
    unsafe {
        let mut token = HANDLE::default();
    }
    Ok(())
}
*/

//fn get_windows_version() -> (u32, u32, u32) {
fn get_windows_version() -> std::result::Result<SerializableOsVersion, &'static str> {
    let mut os_version_info: MaybeUninit<OSVERSIONINFOW> = MaybeUninit::zeroed();
    unsafe {
        (*os_version_info.as_mut_ptr()).dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;

        let status = RtlGetVersion(os_version_info.as_mut_ptr());
        if status.is_ok() {
            let os_version_info = os_version_info.assume_init();
            // doesn't work
            //let colored_json = to_colored_json_auto(&os_version_info).unwrap();
            //debug!("{}", colored_json);
            Ok(SerializableOsVersion{
                major: os_version_info.dwMajorVersion,
                minor: os_version_info.dwMinorVersion,
                pack: 0,
                build: os_version_info.dwBuildNumber,
                //os_version_info.dwPlatformId,
            })
        } else {
            // status.is_err()
            //(0, 0, 0) // In case of error, return 0s
            //(0, 0, 0, 0) // In case of error, return 0s
            Err("RtlGetVersion failed!")
        }
    }

}

// windows-version::OsVersion is not serializable
// So we create our own struct.
#[derive(Serialize)]
struct SerializableOsVersion {
    major: u32,
    minor: u32,
    pack: u32,
    build: u32,
}

impl From<&OsVersion> for SerializableOsVersion {
    fn from(version: &OsVersion) -> Self {
        SerializableOsVersion {
            major: version.major,
            minor: version.minor,
            pack: version.pack,
            build: version.build,
        }
    }
}

pub fn winver() {
    // My version using RtlGetVersion
    //let (major, minor, build) = get_windows_version();
    let version_result = get_windows_version();
    match version_result {
        Ok(version) => {
            let version_json = to_colored_json_auto(&version).unwrap();
            //info!("Windows Version: {}", version_json);
            println!("{}", version_json);
        },
        Err(e) => error!("{}", e),
    }

    // Using windows-version crate
    info!("Current version: {:?}", OsVersion::current());
    let os_version : SerializableOsVersion = (&OsVersion::current()).into();
    let os_version_json = to_colored_json_auto(&os_version).unwrap();
    println!("{}", os_version_json);
    if is_server() {
        info!("Running on a Windows Server release.");
    } else {
        info!("Not running on a Windows Server");
    }
}

pub fn sysinfo() {
    if sysinfo::IS_SUPPORTED_SYSTEM {
        info!("This OS is supported");
    } else {
        info!("This OS isn't supported");
    }

    // Please note that we use "new_all" to ensure that all lists of
    // CPUs and processes are filled!
    let mut sys = System::new_all();

    // First we update all information of our `System` struct.
    sys.refresh_all();

    println!("=> system:");
    // RAM and swap information:
    // trick to get the type information
    //let _: () = sys.total_memory();
    let total_memory = ByteSize(sys.total_memory()).to_string_as(true);
    let used_memory = ByteSize(sys.used_memory()).to_string_as(true);
    let available_memory = ByteSize(sys.available_memory()).to_string_as(true);
    let total_swap = ByteSize(sys.total_swap()).to_string_as(true);
    let used_swap = ByteSize(sys.used_swap()).to_string_as(true);
    info!("total memory: {} bytes", total_memory);
    info!("used memory : {} bytes", used_memory);
    info!("available memory : {} bytes", available_memory);
    info!("total swap  : {} bytes", total_swap);
    info!("used swap   : {} bytes", used_swap);

    // Display system information:
    info!("System name:             {:?}", System::name());
    info!("System kernel version:   {:?}", System::kernel_version());
    info!("System OS version:       {:?}", System::os_version());
    info!("System Long OS version:  {:?}", System::long_os_version());
    info!("System host name:        {:?}", System::host_name());

    // Doesn't work - I think it's a Linux/Unix-like only thing
    /*
    let load_avg = System::load_average();
    println!(
        "one minute: {}%, five minutes: {}%, fifteen minutes: {}%",
        load_avg.one,
        load_avg.five,
        load_avg.fifteen,
    );
    */

    // Number of CPUs:
    info!("=> CPUs:");
    info!("physical core count: {:?}", sys.physical_core_count());
    info!("NB CPUs: {}", sys.cpus().len());

    // Refreshing CPU usage.
    thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_usage();
    for cpu in sys.cpus() {
        info!("{:?} {}% ", cpu, cpu.cpu_usage());
    }

    match sysinfo::get_current_pid() {
        Ok(pid) => {
           info!("current pid: {}", pid);
        }
        Err(e) => {
            error!("failed to get current pid: {}", e);
        }
    }

    /*
    // Display processes ID, name na disk usage:
    for (pid, process) in sys.processes() {
        info!("[{pid}] {:?} {:?}", process.name(), process.disk_usage());
    }
    */

    // We display all disks' information:
    info!("=> disks:");
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        info!("{disk:?}");
    }

    // Network interfaces name, total data received and total data transmitted:
    let networks = Networks::new_with_refreshed_list();
    info!("=> networks:");
    for (interface_name, data) in &networks {
        info!(
            "{interface_name}: {} B (down) / {} B (up)",
            data.total_received(),
            data.total_transmitted(),
        );
        // If you want the amount of data received/transmitted since last call
        // to `Networks::refresh`, use `received`/`transmitted`.
    }

    /*
    // This doesn't work for me - I think it's a Linux-like thing
    // Components temperature:
    let components = Components::new_with_refreshed_list();
    info!("=> components:");
    for component in &components {
        info!("{component:?}");
    }
    */

    info!("=> users:");
    let mut users = Users::new();
    users.refresh_list();
    for user in users.list() {
        println!("{user:?}");
    }

    info!("=> groups:");
    let mut groups = Groups::new();
    groups.refresh_list();
    for group in groups.list() {
        println!("{}", group.name());
    }

}


#[derive(Serialize)]
struct Whoami {
    username: String,
    account: String,
    realname: String,
    // whoami::Arch is not Serialize'able
    //arch: whoami::Arch,
    arch: String,
    // Not serializable
    //desktop_env: whoami::DesktopEnv,
    desktop_env: String,
    hostname: String,
    devicename: String,
    // Not serializable
    //platform: whoami::Platform,
    platform: String,
    distro: String,
    langs: Vec<String>,
}

pub fn whoami() {
    let wai = Whoami{
        username: whoami::username(),
        account: whoami::fallible::account()
            .unwrap_or_else(|_| "<unknown>".to_string()),
        realname: whoami::realname(),
        arch: whoami::arch().to_string(),
        desktop_env: whoami::desktop_env().to_string(),
        hostname: whoami::fallible::hostname()
            .unwrap_or_else(|_| "localhost".to_string()),
        devicename: whoami::devicename(),
        platform: whoami::platform().to_string(),
        distro: whoami::distro(),
        langs: whoami::langs()
            .map(|l| {
                l.map(|l| l.to_string()).collect::<Vec<String>>()
            })
            .unwrap_or_else(|_| vec!["??".to_string()]),
    };
    let json = to_colored_json_auto(&wai).unwrap();
    println!("{}", json);
    // This code works but I think I just want the JSON
    /*
    info!("whoami");
    info!("Username: {}", whoami::username());
    info!("User's Username        whoami::fallible::account():   {}",
        whoami::fallible::account()
            .unwrap_or_else(|_| "<unknown>".to_string()),
    );
    info!("Real (full) name: {}", whoami::realname());
    info!("arch: {}", whoami::arch());
    info!("desktop environment: {}", whoami::desktop_env());
    info!("hostname: {}",
        whoami::fallible::hostname()
            .unwrap_or_else(|_| "localhost".to_string()),
    );
    info!("devicename (pretty name): {}", whoami::devicename());
    info!("platform: {}", whoami::platform());
    info!("distro: {}", whoami::distro());
    info!("User's Langauge(s): {}",
        whoami::langs()
            .map(|l| {
                l.map(|l| l.to_string()).collect::<Vec<String>>().join(", ")
            })
            .unwrap_or_else(|_| "??".to_string()),
    );
    */
}

pub fn privileges() {
    info!("Privileges:");
    let _privs = get_privileges();
}

pub fn keep_awake(active: bool) {
    info!("keep-awake - press Ctrl-C to exit");
    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");
    if active {
        info!("Keeping active by simulating input");
        thread::spawn(|| {
            if let Err(e) = keep_active::simulate_activity() {
                eprintln!("Failed to simulate activity: {}", e);
            }
        });
    }
    let _awake = keep_active::Builder::default()
        .display(true)
        // `ES_SYSTEM_REQUIRED`
        .idle(true)
        // `ES_AWAYMODE_REQUIRED`
        .sleep(true)
        // Used on Linux and macOS
        //.reason("dark-privilege")
        // Used on Linux
        //.app_name("dark-privilege")
        // Used on Linux
        //.app_reverse_domain("io.github.dark-privilege")
        .create();
    loop {
        // Check if we received a signal from the Ctrl-C handler
        if let Ok(_) = rx.recv_timeout(Duration::from_millis(200)) {
            // Exit the loop when Ctrl-C is pressed
            break;
        }
    }
    info!("Ctrl-C detected, exiting");
}
