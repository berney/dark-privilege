// This is code specific to the .dll
// It uses `lib.rs` which is code common to both the .dll and .exe


//use dark_privilege_lib::*;
//use crate::*;


/*
// Macro to export functions with or without arguments
macro_rules! export_fn {
    // Case 1: Functions with no arguments
    ($name:ident ()) => {
        #[no_mangle]
        pub extern "C" fn $name() {
            $name();
        }
    };

    // Case 2: Functions with arguments
    ($name:ident ($($arg:ident : $arg_type:ty),*) -> $ret_type:ty) => {
        #[no_mangle]
        pub extern "C" fn $name($($arg: $arg_type),*) -> $ret_type {
            $name($($arg),*)
        }
    };

    // Case 3: Functions with arguments but no return value (void return type)
    ($name:ident ($($arg:ident : $arg_type:ty),*)) => {
        #[no_mangle]
        pub extern "C" fn $name($($arg: $arg_type),*) {
            $name($($arg),*)
        }
    };
}

// Use the macro to export functions
export_fn!(winver ());
export_fn!(sysinfo ());
*/

#[no_mangle]
//#[export_name = "hello_world"]
pub extern "C" fn hello_world() {
    println!("Hello from Rust DLL!");
}

#[no_mangle]
pub extern "C" fn winver() {
    crate::winver();
}
