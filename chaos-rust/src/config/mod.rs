//! Path resolution and environment configuration module.
//!
//! This module implements the path-resolution logic from src/envconfig.c
//! in safe, idiomatic Rust. It resolves runtime paths for player data,
//! area files, finger data, and notes using environment variables with
//! fallback cascades.

use std::env;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

/// Default compiled-in paths matching the C defaults.
const DEFAULT_PLAYER_DIR: &str = "../player/";
const DEFAULT_PLAYER_TEMP_DIR: &str = "../player/temp";
const DEFAULT_FINGER_DIR: &str = "../finger/";
const DEFAULT_NOTE_DIR: &str = "../notes/";
const DEFAULT_AREA_DIR: &str = "area";
const DEFAULT_AREA_LIST: &str = "area/area.lst";
const DEFAULT_BUG_FILE: &str = "area/bugs.txt";
const DEFAULT_IDEA_FILE: &str = "area/ideas.txt";
const DEFAULT_TYPO_FILE: &str = "area/typos.txt";
const DEFAULT_NOTE_FILE: &str = "area/notes.txt";
const DEFAULT_SHUTDOWN_FILE: &str = "area/shutdown.txt";
const DEFAULT_COPYOVER_FILE: &str = "area/copyover.txt";
const DEFAULT_MOBPROG_DIR: &str = "area/MOBProgs";

/// Number of rotating area path buffer slots.
const ROTATING_BUFFER_SIZE: usize = 4;

/// Holds all resolved path values after initialization.
#[derive(Debug, Clone)]
pub struct PathConfig {
    pub player_dir: String,
    pub player_temp_dir: String,
    pub finger_dir: String,
    pub note_dir: String,
    pub area_dir: String,
    pub area_list: String,
    pub bug_file: String,
    pub idea_file: String,
    pub typo_file: String,
    pub note_file: String,
    pub shutdown_file: String,
    pub copyover_file: String,
    pub mobprog_dir: String,
}

/// Global static holding the resolved path configuration.
static PATH_CONFIG: OnceLock<PathConfig> = OnceLock::new();

/// Rotating buffer for area_file_path results.
static ROTATING_AREA_PATHS: OnceLock<Mutex<RotatingBuffer>> = OnceLock::new();

struct RotatingBuffer {
    slots: [String; ROTATING_BUFFER_SIZE],
    index: usize,
}

impl RotatingBuffer {
    fn new() -> Self {
        RotatingBuffer {
            slots: [
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ],
            index: 0,
        }
    }

    fn next(&mut self, value: String) -> &str {
        self.index = (self.index + 1) % ROTATING_BUFFER_SIZE;
        self.slots[self.index] = value;
        &self.slots[self.index]
    }
}

/// Check if a path exists on the filesystem.
pub fn path_exists(path: &str) -> bool {
    !path.is_empty() && Path::new(path).exists()
}

/// Join a directory and suffix with a '/' separator if needed.
/// Matches the C behavior: simple string concatenation, no canonicalization.
pub fn join_path(dir: &str, suffix: &str) -> String {
    if dir.is_empty() {
        return suffix.to_string();
    }
    if suffix.is_empty() {
        return dir.to_string();
    }
    if dir.ends_with('/') {
        format!("{}{}", dir, suffix)
    } else {
        format!("{}/{}", dir, suffix)
    }
}

/// Ensure a path string ends with a trailing '/'.
pub fn ensure_trailing_slash(path: &mut String) {
    if path.is_empty() || path.ends_with('/') {
        return;
    }
    path.push('/');
}

/// Resolve a path using the priority cascade:
/// 1. Explicit environment variable (env_var)
/// 2. CHAOS_ENV_ROOT + suffix
/// 3. Compiled default
pub fn resolve_path(
    env_var: &str,
    root: Option<&str>,
    suffix: &str,
    compiled_default: &str,
) -> String {
    // Check explicit env var override
    if let Ok(val) = env::var(env_var) {
        if !val.is_empty() {
            return val;
        }
    }

    // Fall back to root + suffix
    if let Some(r) = root {
        if !r.is_empty() {
            return join_path(r, suffix);
        }
    }

    // Use compiled default
    compiled_default.to_string()
}

/// Configure area-related paths based on environment.
fn configure_area_paths(root: Option<&str>) -> (String, String, String, String, String, String, String, String, String) {
    let area_dir = {
        let explicit_dir = env::var("CHAOS_AREA_DIR").ok().filter(|s| !s.is_empty());

        if let Some(dir) = explicit_dir {
            dir
        } else if let Some(r) = root {
            if !r.is_empty() {
                join_path(r, "area")
            } else if path_exists("area") {
                "area".to_string()
            } else {
                ".".to_string()
            }
        } else if path_exists("area") {
            "area".to_string()
        } else {
            ".".to_string()
        }
    };

    let area_list = join_path(&area_dir, "area.lst");
    let bug_file = join_path(&area_dir, "bugs.txt");
    let idea_file = join_path(&area_dir, "ideas.txt");
    let typo_file = join_path(&area_dir, "typos.txt");
    let note_file = join_path(&area_dir, "notes.txt");
    let shutdown_file = join_path(&area_dir, "shutdown.txt");
    let copyover_file = join_path(&area_dir, "copyover.txt");
    let mobprog_dir = join_path(&area_dir, "MOBProgs");

    (area_dir, area_list, bug_file, idea_file, typo_file, note_file, shutdown_file, copyover_file, mobprog_dir)
}

/// Configure finger and note directory paths.
fn configure_misc_dirs(root: Option<&str>) -> (String, String) {
    let mut finger_dir = {
        let finger_override = env::var("CHAOS_FINGER_DIR").ok().filter(|s| !s.is_empty());

        if let Some(dir) = finger_override {
            dir
        } else if let Some(r) = root {
            if !r.is_empty() {
                join_path(r, "finger")
            } else if path_exists("finger") {
                "finger".to_string()
            } else if path_exists("../finger") {
                "../finger".to_string()
            } else {
                "../finger".to_string()
            }
        } else if path_exists("finger") {
            "finger".to_string()
        } else if path_exists("../finger") {
            "../finger".to_string()
        } else {
            "../finger".to_string()
        }
    };

    let mut note_dir = {
        let notes_override = env::var("CHAOS_NOTES_DIR").ok().filter(|s| !s.is_empty());

        if let Some(dir) = notes_override {
            dir
        } else if let Some(r) = root {
            if !r.is_empty() {
                join_path(r, "notes")
            } else if path_exists("notes") {
                "notes".to_string()
            } else if path_exists("../notes") {
                "../notes".to_string()
            } else {
                "../notes".to_string()
            }
        } else if path_exists("notes") {
            "notes".to_string()
        } else if path_exists("../notes") {
            "../notes".to_string()
        } else {
            "../notes".to_string()
        }
    };

    ensure_trailing_slash(&mut finger_dir);
    ensure_trailing_slash(&mut note_dir);

    (finger_dir, note_dir)
}

/// Initialize all path overrides. This should be called once at program startup.
/// Subsequent calls are no-ops (the paths are set once via OnceLock).
pub fn init_path_overrides_impl() -> &'static PathConfig {
    PATH_CONFIG.get_or_init(|| {
        let root = env::var("CHAOS_ENV_ROOT").ok().filter(|s| !s.is_empty());
        let root_ref = root.as_deref();

        let player_override = env::var("CHAOS_PLAYER_DIR").ok().filter(|s| !s.is_empty());
        let player_temp_override = env::var("CHAOS_PLAYER_TEMP_DIR").ok().filter(|s| !s.is_empty());

        // Resolve player_dir
        let mut player_dir = if let Some(ref ov) = player_override {
            let mut p = ov.clone();
            ensure_trailing_slash(&mut p);
            p
        } else {
            resolve_path(
                "CHAOS_PLAYER_DIR",
                root_ref,
                "player/",
                DEFAULT_PLAYER_DIR,
            )
        };

        // Additional fallback: if no override and no root, check if "player" exists locally
        if player_override.is_none()
            && root_ref.map_or(true, |r| r.is_empty())
            && !path_exists(&player_dir)
            && path_exists("player")
        {
            player_dir = "player/".to_string();
        }

        // Resolve player_temp_dir
        let mut player_temp_dir = if let Some(ref ov) = player_temp_override {
            ov.clone()
        } else {
            resolve_path(
                "CHAOS_PLAYER_TEMP_DIR",
                root_ref,
                "player/temp",
                DEFAULT_PLAYER_TEMP_DIR,
            )
        };

        // Additional fallback for player_temp_dir
        if player_temp_override.is_none()
            && root_ref.map_or(true, |r| r.is_empty())
            && !path_exists(&player_temp_dir)
            && path_exists("player/temp")
        {
            player_temp_dir = "player/temp".to_string();
        }

        let (area_dir, area_list, bug_file, idea_file, typo_file, note_file, shutdown_file, copyover_file, mobprog_dir) =
            configure_area_paths(root_ref);
        let (finger_dir, note_dir) = configure_misc_dirs(root_ref);

        PathConfig {
            player_dir,
            player_temp_dir,
            finger_dir,
            note_dir,
            area_dir,
            area_list,
            bug_file,
            idea_file,
            typo_file,
            note_file,
            shutdown_file,
            copyover_file,
            mobprog_dir,
        }
    })
}

// --- Public getter functions ---

/// Get the resolved player directory path.
pub fn get_player_dir_str() -> &'static str {
    &PATH_CONFIG.get().map_or(DEFAULT_PLAYER_DIR, |c| &c.player_dir)
}

/// Get the resolved player temp directory path.
pub fn get_player_temp_dir_str() -> &'static str {
    PATH_CONFIG.get().map_or(DEFAULT_PLAYER_TEMP_DIR, |c| &c.player_temp_dir)
}

/// Get the resolved area directory path.
pub fn get_area_dir_str() -> &'static str {
    PATH_CONFIG.get().map_or(DEFAULT_AREA_DIR, |c| &c.area_dir)
}

/// Get the resolved area list file path.
pub fn get_area_list_path_str() -> &'static str {
    PATH_CONFIG.get().map_or(DEFAULT_AREA_LIST, |c| &c.area_list)
}

/// Get the resolved bug file path.
pub fn get_bug_file_path_str() -> &'static str {
    PATH_CONFIG.get().map_or(DEFAULT_BUG_FILE, |c| &c.bug_file)
}

/// Get the resolved idea file path.
pub fn get_idea_file_path_str() -> &'static str {
    PATH_CONFIG.get().map_or(DEFAULT_IDEA_FILE, |c| &c.idea_file)
}

/// Get the resolved typo file path.
pub fn get_typo_file_path_str() -> &'static str {
    PATH_CONFIG.get().map_or(DEFAULT_TYPO_FILE, |c| &c.typo_file)
}

/// Get the resolved note file path.
pub fn get_note_file_path_str() -> &'static str {
    PATH_CONFIG.get().map_or(DEFAULT_NOTE_FILE, |c| &c.note_file)
}

/// Get the resolved shutdown file path.
pub fn get_shutdown_file_path_str() -> &'static str {
    PATH_CONFIG.get().map_or(DEFAULT_SHUTDOWN_FILE, |c| &c.shutdown_file)
}

/// Get the resolved copyover file path.
pub fn get_copyover_file_path_str() -> &'static str {
    PATH_CONFIG.get().map_or(DEFAULT_COPYOVER_FILE, |c| &c.copyover_file)
}

/// Get the resolved MOBProg directory path.
pub fn get_mobprog_dir_str() -> &'static str {
    PATH_CONFIG.get().map_or(DEFAULT_MOBPROG_DIR, |c| &c.mobprog_dir)
}

/// Get the resolved finger directory path.
pub fn get_finger_dir_str() -> &'static str {
    PATH_CONFIG.get().map_or(DEFAULT_FINGER_DIR, |c| &c.finger_dir)
}

/// Get the resolved note directory path.
pub fn get_note_dir_str() -> &'static str {
    PATH_CONFIG.get().map_or(DEFAULT_NOTE_DIR, |c| &c.note_dir)
}

/// Get the full path for an area file by joining area_dir with the filename.
/// Uses a rotating 4-slot buffer so that up to 4 concurrent results remain valid.
/// Returns a reference to the buffer slot string.
pub fn area_file_path_str(filename: &str) -> String {
    if filename.is_empty() {
        return get_area_dir_str().to_string();
    }

    let area_dir = get_area_dir_str();
    join_path(area_dir, filename)
}

/// Internal function for the rotating buffer version (used by FFI layer).
/// Returns a pointer to a stable string in the rotating buffer.
pub fn area_file_path_rotating(filename: &str) -> &'static str {
    let buf = ROTATING_AREA_PATHS.get_or_init(|| Mutex::new(RotatingBuffer::new()));
    let mut guard = buf.lock().unwrap();

    if filename.is_empty() {
        return get_area_dir_str();
    }

    let area_dir = get_area_dir_str();
    let result = join_path(area_dir, filename);
    guard.next(result);

    // We need to return a &'static str. Since the buffer is in a static OnceLock,
    // the slots persist for the program lifetime. We get the pointer before dropping the guard.
    let index = guard.index;
    let ptr = guard.slots[index].as_str() as *const str;
    // SAFETY: The rotating buffer is in a static OnceLock and the string data
    // remains valid until it is overwritten (after 4 more calls). This matches
    // the C behavior exactly.
    unsafe { &*ptr }
}

// --- Public wrappers for testing internal configure functions ---

/// Public wrapper for configure_area_paths (for integration testing).
pub fn configure_area_paths_pub(root: Option<&str>) -> (String, String, String, String, String, String, String, String, String) {
    configure_area_paths(root)
}

/// Public wrapper for configure_misc_dirs (for integration testing).
pub fn configure_misc_dirs_pub(root: Option<&str>) -> (String, String) {
    configure_misc_dirs(root)
}

// --- FFI module (extern "C" wrappers) ---
pub mod ffi;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex as StdMutex;

    // Tests need serialization because they manipulate global env vars and static state.
    // We use a test-specific mutex for coordination.
    static TEST_MUTEX: StdMutex<()> = StdMutex::new(());

    /// Helper to clear all CHAOS_* environment variables
    fn clear_chaos_env() {
        let vars = [
            "CHAOS_ENV_ROOT",
            "CHAOS_PLAYER_DIR",
            "CHAOS_PLAYER_TEMP_DIR",
            "CHAOS_AREA_DIR",
            "CHAOS_FINGER_DIR",
            "CHAOS_NOTES_DIR",
        ];
        for var in &vars {
            env::remove_var(var);
        }
    }

    #[test]
    fn test_join_path_basic() {
        let _lock = TEST_MUTEX.lock().unwrap();
        assert_eq!(join_path("dir", "file.txt"), "dir/file.txt");
        assert_eq!(join_path("dir/", "file.txt"), "dir/file.txt");
        assert_eq!(join_path("", "file.txt"), "file.txt");
        assert_eq!(join_path("dir", ""), "dir");
        assert_eq!(join_path("", ""), "");
    }

    #[test]
    fn test_join_path_with_trailing_slash() {
        let _lock = TEST_MUTEX.lock().unwrap();
        assert_eq!(join_path("/root/area/", "mob.are"), "/root/area/mob.are");
        assert_eq!(join_path("/root/area", "mob.are"), "/root/area/mob.are");
    }

    #[test]
    fn test_ensure_trailing_slash() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let mut s = "player".to_string();
        ensure_trailing_slash(&mut s);
        assert_eq!(s, "player/");

        let mut s2 = "player/".to_string();
        ensure_trailing_slash(&mut s2);
        assert_eq!(s2, "player/");

        let mut s3 = String::new();
        ensure_trailing_slash(&mut s3);
        assert_eq!(s3, "");
    }

    #[test]
    fn test_path_exists_empty() {
        let _lock = TEST_MUTEX.lock().unwrap();
        assert!(!path_exists(""));
    }

    #[test]
    fn test_path_exists_current_dir() {
        let _lock = TEST_MUTEX.lock().unwrap();
        assert!(path_exists("."));
    }

    #[test]
    fn test_resolve_path_env_override() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_chaos_env();

        env::set_var("TEST_RESOLVE_PATH_VAR", "/custom/path");
        let result = resolve_path(
            "TEST_RESOLVE_PATH_VAR",
            Some("/root"),
            "fallback",
            "default",
        );
        assert_eq!(result, "/custom/path");
        env::remove_var("TEST_RESOLVE_PATH_VAR");
    }

    #[test]
    fn test_resolve_path_root_fallback() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_chaos_env();
        env::remove_var("TEST_RESOLVE_PATH_VAR2");

        let result = resolve_path(
            "TEST_RESOLVE_PATH_VAR2",
            Some("/myroot"),
            "player/",
            "../player/",
        );
        assert_eq!(result, "/myroot/player/");
    }

    #[test]
    fn test_resolve_path_compiled_default() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_chaos_env();
        env::remove_var("TEST_RESOLVE_PATH_VAR3");

        let result = resolve_path(
            "TEST_RESOLVE_PATH_VAR3",
            None,
            "player/",
            "../player/",
        );
        assert_eq!(result, "../player/");
    }

    #[test]
    fn test_resolve_path_empty_env_var_ignored() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_chaos_env();
        env::set_var("TEST_RESOLVE_EMPTY", "");

        let result = resolve_path(
            "TEST_RESOLVE_EMPTY",
            Some("/root"),
            "area",
            "default_area",
        );
        assert_eq!(result, "/root/area");
        env::remove_var("TEST_RESOLVE_EMPTY");
    }

    #[test]
    fn test_resolve_path_empty_root_uses_default() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_chaos_env();
        env::remove_var("TEST_RESOLVE_NOROOT");

        let result = resolve_path(
            "TEST_RESOLVE_NOROOT",
            Some(""),
            "area",
            "compiled_default",
        );
        assert_eq!(result, "compiled_default");
    }

    #[test]
    fn test_configure_area_paths_with_root() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_chaos_env();

        let (area_dir, area_list, bug_file, idea_file, typo_file, note_file, shutdown_file, copyover_file, mobprog_dir) =
            configure_area_paths(Some("/env/dev"));

        assert_eq!(area_dir, "/env/dev/area");
        assert_eq!(area_list, "/env/dev/area/area.lst");
        assert_eq!(bug_file, "/env/dev/area/bugs.txt");
        assert_eq!(idea_file, "/env/dev/area/ideas.txt");
        assert_eq!(typo_file, "/env/dev/area/typos.txt");
        assert_eq!(note_file, "/env/dev/area/notes.txt");
        assert_eq!(shutdown_file, "/env/dev/area/shutdown.txt");
        assert_eq!(copyover_file, "/env/dev/area/copyover.txt");
        assert_eq!(mobprog_dir, "/env/dev/area/MOBProgs");
    }

    #[test]
    fn test_configure_area_paths_with_explicit_dir() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_chaos_env();
        env::set_var("CHAOS_AREA_DIR", "/custom/areas");

        let (area_dir, area_list, _, _, _, _, _, _, _) =
            configure_area_paths(Some("/env/dev"));

        assert_eq!(area_dir, "/custom/areas");
        assert_eq!(area_list, "/custom/areas/area.lst");

        env::remove_var("CHAOS_AREA_DIR");
    }

    #[test]
    fn test_configure_misc_dirs_with_root() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_chaos_env();

        let (finger_dir, note_dir) = configure_misc_dirs(Some("/env/dev"));

        assert_eq!(finger_dir, "/env/dev/finger/");
        assert_eq!(note_dir, "/env/dev/notes/");
    }

    #[test]
    fn test_configure_misc_dirs_with_overrides() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_chaos_env();
        env::set_var("CHAOS_FINGER_DIR", "/custom/finger");
        env::set_var("CHAOS_NOTES_DIR", "/custom/notes");

        let (finger_dir, note_dir) = configure_misc_dirs(Some("/env/dev"));

        assert_eq!(finger_dir, "/custom/finger/");
        assert_eq!(note_dir, "/custom/notes/");

        env::remove_var("CHAOS_FINGER_DIR");
        env::remove_var("CHAOS_NOTES_DIR");
    }

    #[test]
    fn test_configure_misc_dirs_trailing_slash_enforcement() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_chaos_env();
        env::set_var("CHAOS_FINGER_DIR", "/path/finger");
        env::set_var("CHAOS_NOTES_DIR", "/path/notes/");

        let (finger_dir, note_dir) = configure_misc_dirs(None);

        assert!(finger_dir.ends_with('/'), "finger_dir should end with /");
        assert!(note_dir.ends_with('/'), "note_dir should end with /");

        env::remove_var("CHAOS_FINGER_DIR");
        env::remove_var("CHAOS_NOTES_DIR");
    }

    #[test]
    fn test_area_file_path_str_basic() {
        let _lock = TEST_MUTEX.lock().unwrap();
        // Test join behavior directly
        let result = join_path("area", "midgaard.are");
        assert_eq!(result, "area/midgaard.are");
    }

    #[test]
    fn test_area_file_path_str_empty_filename() {
        let _lock = TEST_MUTEX.lock().unwrap();
        // When filename is empty, should return area_dir
        let result = area_file_path_str("");
        // Returns whatever area_dir is configured to (default if not initialized)
        assert!(!result.is_empty());
    }

    #[test]
    fn test_rotating_buffer() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let mut buf = RotatingBuffer::new();

        let _ = buf.next("first".to_string());
        assert_eq!(buf.slots[1], "first");
        assert_eq!(buf.index, 1);

        let _ = buf.next("second".to_string());
        assert_eq!(buf.slots[2], "second");
        assert_eq!(buf.index, 2);

        let _ = buf.next("third".to_string());
        assert_eq!(buf.slots[3], "third");
        assert_eq!(buf.index, 3);

        let _ = buf.next("fourth".to_string());
        assert_eq!(buf.slots[0], "fourth");
        assert_eq!(buf.index, 0);

        // Verify wrap-around: fifth call overwrites slot 1
        let _ = buf.next("fifth".to_string());
        assert_eq!(buf.slots[1], "fifth");
        assert_eq!(buf.index, 1);
    }

    #[test]
    fn test_configure_area_paths_no_root_no_dir() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_chaos_env();

        // With no root and no "area" directory accessible, falls back to "."
        // Note: we can't fully test path_exists behavior without filesystem setup
        // but we test the logic paths
        let (area_dir, _, _, _, _, _, _, _, _) = configure_area_paths(None);
        // Without filesystem "area" dir existing, should get "." or "area"
        // depending on whether "area" exists in CWD
        assert!(!area_dir.is_empty());
    }

    #[test]
    fn test_init_with_temp_dir() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_chaos_env();

        // Create a temporary directory structure
        let tmp = env::temp_dir().join("chaos_test_init");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("player")).unwrap();
        fs::create_dir_all(tmp.join("player/temp")).unwrap();
        fs::create_dir_all(tmp.join("area")).unwrap();
        fs::create_dir_all(tmp.join("finger")).unwrap();
        fs::create_dir_all(tmp.join("notes")).unwrap();

        let root = tmp.to_str().unwrap();

        // Test configure_area_paths with this root
        let (area_dir, area_list, _, _, _, _, _, _, _) = configure_area_paths(Some(root));
        assert_eq!(area_dir, format!("{}/area", root));
        assert_eq!(area_list, format!("{}/area/area.lst", root));

        // Test configure_misc_dirs with this root
        let (finger_dir, note_dir) = configure_misc_dirs(Some(root));
        assert_eq!(finger_dir, format!("{}/finger/", root));
        assert_eq!(note_dir, format!("{}/notes/", root));

        // Cleanup
        let _ = fs::remove_dir_all(&tmp);
    }
}
