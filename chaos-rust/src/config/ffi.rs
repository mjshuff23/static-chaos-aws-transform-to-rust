//! FFI boundary functions for C interop.
//!
//! This module contains the #[no_mangle] pub extern "C" functions
//! matching the signatures declared in src/merc.h (lines 111-125).
//! These functions return *const c_char pointers that remain valid
//! for the lifetime of the program.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;
use std::sync::OnceLock;

use super::{area_file_path_rotating, get_area_dir_str, get_area_list_path_str,
            get_bug_file_path_str, get_copyover_file_path_str, get_finger_dir_str,
            get_idea_file_path_str, get_mobprog_dir_str, get_note_dir_str,
            get_note_file_path_str, get_player_dir_str, get_player_temp_dir_str,
            get_shutdown_file_path_str, get_typo_file_path_str, init_path_overrides_impl};

/// Static cache of CStrings so that returned pointers remain valid for program lifetime.
static FFI_CACHE: OnceLock<Mutex<Vec<CString>>> = OnceLock::new();

/// Get or initialize the FFI cache.
fn cache() -> &'static Mutex<Vec<CString>> {
    FFI_CACHE.get_or_init(|| Mutex::new(Vec::new()))
}

/// Cache a Rust string and return a *const c_char that is valid for the program lifetime.
fn cache_str(s: &str) -> *const c_char {
    let c_string = CString::new(s).unwrap_or_else(|_| CString::new("").unwrap());
    let mut guard = cache().lock().unwrap();
    guard.push(c_string);
    guard.last().unwrap().as_ptr()
}

/// Rotating buffer for area_file_path FFI results.
/// We keep 4 CStrings alive at a time matching the C behavior.
static AREA_FILE_CACHE: OnceLock<Mutex<AreaFileCache>> = OnceLock::new();

struct AreaFileCache {
    slots: [Option<CString>; 4],
    index: usize,
}

impl AreaFileCache {
    fn new() -> Self {
        AreaFileCache {
            slots: [None, None, None, None],
            index: 0,
        }
    }

    fn next(&mut self, value: &str) -> *const c_char {
        self.index = (self.index + 1) % 4;
        let c_string = CString::new(value).unwrap_or_else(|_| CString::new("").unwrap());
        let ptr = c_string.as_ptr();
        self.slots[self.index] = Some(c_string);
        ptr
    }
}

fn area_file_cache() -> &'static Mutex<AreaFileCache> {
    AREA_FILE_CACHE.get_or_init(|| Mutex::new(AreaFileCache::new()))
}

// --- FFI exports matching src/merc.h lines 111-125 ---

#[no_mangle]
pub extern "C" fn init_path_overrides() {
    init_path_overrides_impl();
}

#[no_mangle]
pub extern "C" fn get_player_dir() -> *const c_char {
    cache_str(get_player_dir_str())
}

#[no_mangle]
pub extern "C" fn get_player_temp_dir() -> *const c_char {
    cache_str(get_player_temp_dir_str())
}

#[no_mangle]
pub extern "C" fn get_area_dir() -> *const c_char {
    cache_str(get_area_dir_str())
}

#[no_mangle]
pub extern "C" fn get_area_list_path() -> *const c_char {
    cache_str(get_area_list_path_str())
}

#[no_mangle]
pub extern "C" fn get_bug_file_path() -> *const c_char {
    cache_str(get_bug_file_path_str())
}

#[no_mangle]
pub extern "C" fn get_idea_file_path() -> *const c_char {
    cache_str(get_idea_file_path_str())
}

#[no_mangle]
pub extern "C" fn get_typo_file_path() -> *const c_char {
    cache_str(get_typo_file_path_str())
}

#[no_mangle]
pub extern "C" fn get_note_file_path() -> *const c_char {
    cache_str(get_note_file_path_str())
}

#[no_mangle]
pub extern "C" fn get_shutdown_file_path() -> *const c_char {
    cache_str(get_shutdown_file_path_str())
}

#[no_mangle]
pub extern "C" fn get_copyover_file_path() -> *const c_char {
    cache_str(get_copyover_file_path_str())
}

#[no_mangle]
pub extern "C" fn get_mobprog_dir() -> *const c_char {
    cache_str(get_mobprog_dir_str())
}

#[no_mangle]
pub extern "C" fn get_finger_dir() -> *const c_char {
    cache_str(get_finger_dir_str())
}

#[no_mangle]
pub extern "C" fn get_note_dir() -> *const c_char {
    cache_str(get_note_dir_str())
}

#[no_mangle]
pub extern "C" fn area_file_path(filename: *const c_char) -> *const c_char {
    // SAFETY: The filename pointer comes from C and must be a valid null-terminated string.
    // This is the only unsafe block in the FFI boundary, required to read the C string.
    let fname = if filename.is_null() {
        ""
    } else {
        unsafe { CStr::from_ptr(filename) }
            .to_str()
            .unwrap_or("")
    };

    let result = area_file_path_rotating(fname);
    let mut guard = area_file_cache().lock().unwrap();
    guard.next(result)
}
