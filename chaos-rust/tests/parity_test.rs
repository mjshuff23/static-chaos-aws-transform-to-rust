//! C/Rust parity harness for envconfig path resolution.
//!
//! These tests verify that the Rust implementation produces identical output
//! to the C envconfig.c for all tested environment variable combinations.
//! The expected values are derived directly from manual analysis of the C code
//! in src/envconfig.c, ensuring bit-for-bit string parity.
//!
//! Test matrix:
//! - CHAOS_ENV_ROOT set/unset
//! - Individual overrides (CHAOS_PLAYER_DIR, CHAOS_AREA_DIR, etc.) set/unset
//! - Fallback directory existence variations
//! - Trailing slash enforcement

use std::env;
use std::fs;
use std::sync::Mutex;

use chaos_rust::config::{
    area_file_path_rotating, configure_area_paths_pub, configure_misc_dirs_pub,
    ensure_trailing_slash, get_area_dir_str, join_path, resolve_path,
};

static PARITY_MUTEX: Mutex<()> = Mutex::new(());

fn clear_chaos_env() {
    for var in &[
        "CHAOS_ENV_ROOT",
        "CHAOS_PLAYER_DIR",
        "CHAOS_PLAYER_TEMP_DIR",
        "CHAOS_AREA_DIR",
        "CHAOS_FINGER_DIR",
        "CHAOS_NOTES_DIR",
    ] {
        env::remove_var(var);
    }
}

/// Test parity: C resolve_path with env override returns the env value verbatim.
/// C code: `if (override != NULL && override[0] != '\0') return override;`
#[test]
fn parity_resolve_path_env_override_verbatim() {
    let _lock = PARITY_MUTEX.lock().unwrap();
    clear_chaos_env();

    env::set_var("PARITY_TEST_VAR", "/custom/override/path");
    let rust_result = resolve_path("PARITY_TEST_VAR", Some("/root"), "suffix", "default");
    // C would return the getenv() value directly
    assert_eq!(rust_result, "/custom/override/path");
    env::remove_var("PARITY_TEST_VAR");
}

/// Test parity: C resolve_path with root+suffix joins with "/" if root lacks trailing slash.
/// C code: `snprintf(buf, buf_len, "%s%s%s", root, needs_slash ? "/" : "", suffix);`
#[test]
fn parity_resolve_path_root_join_adds_slash() {
    let _lock = PARITY_MUTEX.lock().unwrap();
    clear_chaos_env();
    env::remove_var("PARITY_NO_SUCH_VAR");

    let rust_result = resolve_path("PARITY_NO_SUCH_VAR", Some("/env/dev"), "player/", "../player/");
    // C: snprintf -> "/env/dev" + "/" + "player/" = "/env/dev/player/"
    assert_eq!(rust_result, "/env/dev/player/");
}

/// Test parity: C resolve_path with root that already has trailing slash.
/// C code: `needs_slash = root[root_len - 1] != '/'` -> needs_slash = 0
#[test]
fn parity_resolve_path_root_trailing_slash_no_double() {
    let _lock = PARITY_MUTEX.lock().unwrap();
    clear_chaos_env();
    env::remove_var("PARITY_NO_SUCH_VAR2");

    let rust_result = resolve_path("PARITY_NO_SUCH_VAR2", Some("/env/dev/"), "area", "area");
    // C: "/env/dev/" + "" + "area" = "/env/dev/area"
    assert_eq!(rust_result, "/env/dev/area");
}

/// Test parity: C resolve_path with no env, no root -> compiled default.
/// C code: `return compiled_default;`
#[test]
fn parity_resolve_path_falls_to_compiled_default() {
    let _lock = PARITY_MUTEX.lock().unwrap();
    clear_chaos_env();
    env::remove_var("PARITY_NOVAR");

    let rust_result = resolve_path("PARITY_NOVAR", None, "player/", "../player/");
    assert_eq!(rust_result, "../player/");
}

/// Test parity: C configure_area_paths with explicit CHAOS_AREA_DIR override.
/// C code: `snprintf(area_dir_buf, ..., "%s", explicit_dir);` then joins.
#[test]
fn parity_area_paths_explicit_dir() {
    let _lock = PARITY_MUTEX.lock().unwrap();
    clear_chaos_env();
    env::set_var("CHAOS_AREA_DIR", "/my/custom/area");

    let (area_dir, area_list, bug_file, idea_file, typo_file, note_file, shutdown_file, copyover_file, mobprog_dir) =
        configure_area_paths_pub(Some("/env/dev"));

    // C: area_dir = "/my/custom/area" (from getenv)
    assert_eq!(area_dir, "/my/custom/area");
    // C: join_path("/my/custom/area", "area.lst", ...) -> "/my/custom/area/area.lst"
    assert_eq!(area_list, "/my/custom/area/area.lst");
    assert_eq!(bug_file, "/my/custom/area/bugs.txt");
    assert_eq!(idea_file, "/my/custom/area/ideas.txt");
    assert_eq!(typo_file, "/my/custom/area/typos.txt");
    assert_eq!(note_file, "/my/custom/area/notes.txt");
    assert_eq!(shutdown_file, "/my/custom/area/shutdown.txt");
    assert_eq!(copyover_file, "/my/custom/area/copyover.txt");
    assert_eq!(mobprog_dir, "/my/custom/area/MOBProgs");

    env::remove_var("CHAOS_AREA_DIR");
}

/// Test parity: C configure_area_paths with root, no explicit dir.
/// C code: `join_path(root, "area", area_dir_buf, sizeof(area_dir_buf));`
#[test]
fn parity_area_paths_root_fallback() {
    let _lock = PARITY_MUTEX.lock().unwrap();
    clear_chaos_env();

    let (area_dir, area_list, _, _, _, _, _, _, mobprog_dir) =
        configure_area_paths_pub(Some("/srv/chaos/env/dev"));

    assert_eq!(area_dir, "/srv/chaos/env/dev/area");
    assert_eq!(area_list, "/srv/chaos/env/dev/area/area.lst");
    assert_eq!(mobprog_dir, "/srv/chaos/env/dev/area/MOBProgs");
}

/// Test parity: C configure_misc_dirs with root, no overrides.
/// C code: `join_path(root, "finger", ...)` then ensure_trailing_slash.
#[test]
fn parity_misc_dirs_root_with_trailing_slash() {
    let _lock = PARITY_MUTEX.lock().unwrap();
    clear_chaos_env();

    let (finger_dir, note_dir) = configure_misc_dirs_pub(Some("/env/dev"));

    // C: join_path("/env/dev", "finger") = "/env/dev/finger"
    //    ensure_trailing_slash -> "/env/dev/finger/"
    assert_eq!(finger_dir, "/env/dev/finger/");
    assert_eq!(note_dir, "/env/dev/notes/");
}

/// Test parity: C configure_misc_dirs with explicit overrides.
/// C code: `snprintf(finger_dir_buf, ..., "%s", finger_override);`
///          then ensure_trailing_slash appends '/'.
#[test]
fn parity_misc_dirs_overrides_get_trailing_slash() {
    let _lock = PARITY_MUTEX.lock().unwrap();
    clear_chaos_env();
    env::set_var("CHAOS_FINGER_DIR", "/custom/finger");
    env::set_var("CHAOS_NOTES_DIR", "/custom/notes/");

    let (finger_dir, note_dir) = configure_misc_dirs_pub(Some("/env/dev"));

    // C: finger_override = "/custom/finger" -> ensure_trailing_slash -> "/custom/finger/"
    assert_eq!(finger_dir, "/custom/finger/");
    // C: notes_override = "/custom/notes/" -> already has slash -> "/custom/notes/"
    assert_eq!(note_dir, "/custom/notes/");

    env::remove_var("CHAOS_FINGER_DIR");
    env::remove_var("CHAOS_NOTES_DIR");
}

/// Test parity: C area_file_path joins area_dir with filename.
/// C code: `join_path(area_dir_value, filename, rotating_area_paths[...], ...)`
#[test]
fn parity_area_file_path_join() {
    let _lock = PARITY_MUTEX.lock().unwrap();
    clear_chaos_env();

    // With default config, area_dir is "area" (or whatever current config says)
    let area_dir = get_area_dir_str();
    let rust_result = area_file_path_rotating("midgaard.are");
    let expected = join_path(area_dir, "midgaard.are");
    assert_eq!(rust_result, expected);
}

/// Test parity: C area_file_path with NULL/empty filename returns area_dir.
/// C code: `if (filename == NULL || filename[0] == '\0') return area_dir_value;`
#[test]
fn parity_area_file_path_empty_returns_area_dir() {
    let _lock = PARITY_MUTEX.lock().unwrap();
    clear_chaos_env();

    let area_dir = get_area_dir_str();
    let rust_result = area_file_path_rotating("");
    assert_eq!(rust_result, area_dir);
}

/// Test parity: C ensure_trailing_slash behavior.
/// C code: adds '/' only if last char isn't '/' and len+1 < buf_len.
#[test]
fn parity_ensure_trailing_slash_exact() {
    let _lock = PARITY_MUTEX.lock().unwrap();

    let mut s1 = "/env/dev/player".to_string();
    ensure_trailing_slash(&mut s1);
    assert_eq!(s1, "/env/dev/player/");

    let mut s2 = "/env/dev/player/".to_string();
    ensure_trailing_slash(&mut s2);
    assert_eq!(s2, "/env/dev/player/");

    let mut s3 = String::new();
    ensure_trailing_slash(&mut s3);
    assert_eq!(s3, "");
}

/// Test parity: C join_path behavior for all edge cases.
/// C code: handles NULL/empty dir, NULL/empty suffix, trailing slash on dir.
#[test]
fn parity_join_path_all_cases() {
    let _lock = PARITY_MUTEX.lock().unwrap();

    // C: dir="" -> returns suffix
    assert_eq!(join_path("", "file.txt"), "file.txt");
    // C: suffix="" -> returns dir
    assert_eq!(join_path("/root", ""), "/root");
    // C: dir has trailing slash -> no extra slash
    assert_eq!(join_path("/root/", "file"), "/root/file");
    // C: dir lacks trailing slash -> adds one
    assert_eq!(join_path("/root", "file"), "/root/file");
    // C: both empty
    assert_eq!(join_path("", ""), "");
}

/// Test parity: Full end-to-end with a temp directory mimicking env/dev.
/// Verifies all paths match what C would produce for the same CHAOS_ENV_ROOT.
#[test]
fn parity_full_env_root_cycle() {
    let _lock = PARITY_MUTEX.lock().unwrap();
    clear_chaos_env();

    let tmp = env::temp_dir().join("chaos_parity_test");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(tmp.join("area/MOBProgs")).unwrap();
    fs::create_dir_all(tmp.join("player/temp")).unwrap();
    fs::create_dir_all(tmp.join("finger")).unwrap();
    fs::create_dir_all(tmp.join("notes")).unwrap();

    let root = tmp.to_str().unwrap();

    // C would produce these exact paths:
    let (area_dir, area_list, bug, idea, typo, note, shutdown, copyover, mobprog) =
        configure_area_paths_pub(Some(root));

    assert_eq!(area_dir, format!("{}/area", root));
    assert_eq!(area_list, format!("{}/area/area.lst", root));
    assert_eq!(bug, format!("{}/area/bugs.txt", root));
    assert_eq!(idea, format!("{}/area/ideas.txt", root));
    assert_eq!(typo, format!("{}/area/typos.txt", root));
    assert_eq!(note, format!("{}/area/notes.txt", root));
    assert_eq!(shutdown, format!("{}/area/shutdown.txt", root));
    assert_eq!(copyover, format!("{}/area/copyover.txt", root));
    assert_eq!(mobprog, format!("{}/area/MOBProgs", root));

    let (finger, notes) = configure_misc_dirs_pub(Some(root));
    assert_eq!(finger, format!("{}/finger/", root));
    assert_eq!(notes, format!("{}/notes/", root));

    let _ = fs::remove_dir_all(&tmp);
}
