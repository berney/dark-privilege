//use std::default::Default;
//use std::path::PathBuf;
use std::mem::MaybeUninit;
//use std::thread;
//use std::time::Duration;
use std::io::{self, Read};

use env_logger::Env;

//use log::{debug, error, log_enabled, info, Level};
use log::{debug, error, info};

use clap::{CommandFactory, Parser, Subcommand};

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

use enigo::{
    Button,
    //Direction::{Click, Press, Release},
    Direction::Click,
    Enigo, Mouse,
    //Key,
    Keyboard, Settings,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    ///// Optional name to operate on
    //name: Option<String>,

    ///// Sets a custom config file
    //#[arg(short, long, value_name = "FILE")]
    //config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    //#[arg(short, long)]
    //http: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Who Am I?
    Whoami {
    },

    /// Get Windows Version
    Winver {
    },

    /// List privileges
    Privileges {
    },

    /// All info collection (everything except `paste`)
    All {
    },

    /// Clicks the mouse and types a string into the window under the cursor
    /// Useful for locked down RDP and Citrix with clipboard disabled
    Paste {
        message: Option<Vec<String>>,
    },

    /// Prints version information
    Version,
}


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


fn paste(message : &String) {
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

fn winver() {
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

fn whoami() {
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

fn privileges() {
    info!("Privileges:");
    let _privs = get_privileges();
}


fn main() {
    // Defaults to `error` level
    //env_logger::init();
    // Default to `info` level if one isn't specified
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();
    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match cli.debug {
        0 => debug!("Debug mode is off"),
        1 => debug!("Debug mode is kind of on"),
        2 => debug!("Debug mode is on"),
        _ => debug!("Don't be crazy"),
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Winver {}) => {
            winver();
        }
        Some(Commands::Whoami {}) => {
            whoami();
        }
        Some(Commands::Privileges {}) => {
            privileges();
        }
        Some(Commands::All {}) => {
            winver();
            whoami();
            privileges();
        }
        Some(Commands::Paste { message }) => {
            let msg = if let Some(words) = message {
                // message argument given
                words.join(" ")
            } else {
                // use stdin
                debug!("Using stdin");
                let mut input = String::new();
                io::stdin().read_to_string(&mut input).expect("Failed to read from stdin");
                if input.is_empty() {
                    debug!("No input provided via stdin");
                }

                input
            };
            debug!("msg: {}", msg);
            paste(&msg);
        }
        Some(Commands::Version) => {
            println!("{}", Cli::command().render_version());
        }
        None => {}
    }
}
