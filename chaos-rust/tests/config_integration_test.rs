//! Integration tests for the config module.
//!
//! These tests verify the full init/getter cycle through the public Rust API
//! using temporary directory structures that mimic the runtime environment.

use std::env;
use std::fs;
use std::sync::Mutex;

use chaos_rust::config::{
    configure_area_paths_pub, configure_misc_dirs_pub, ensure_trailing_slash, join_path,
    path_exists, resolve_path,
};

// Serialize integration tests since they manipulate environment variables.
static INTEGRATION_MUTEX: Mutex<()> = Mutex::new(());

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
fn test_full_init_cycle_with_env_root() {
    let _lock = INTEGRATION_MUTEX.lock().unwrap();
    clear_chaos_env();

    // Create a temporary directory structure mimicking env/dev/
    let tmp = env::temp_dir().join("chaos_integration_test_1");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(tmp.join("player/temp")).unwrap();
    fs::create_dir_all(tmp.join("area/MOBProgs")).unwrap();
    fs::create_dir_all(tmp.join("finger")).unwrap();
    fs::create_dir_all(tmp.join("notes")).unwrap();

    // Create area.lst file
    fs::write(tmp.join("area/area.lst"), "midgaard.are\n").unwrap();

    let root = tmp.to_str().unwrap();

    // Test resolve_path with root
    let player_dir = resolve_path("CHAOS_PLAYER_DIR", Some(root), "player/", "../player/");
    assert_eq!(player_dir, format!("{}/player/", root));

    let player_temp = resolve_path("CHAOS_PLAYER_TEMP_DIR", Some(root), "player/temp", "../player/temp");
    assert_eq!(player_temp, format!("{}/player/temp", root));

    // Test configure_area_paths
    env::set_var("CHAOS_ENV_ROOT", root);
    let (area_dir, area_list, bug_file, idea_file, typo_file, note_file, shutdown_file, copyover_file, mobprog_dir) =
        configure_area_paths_pub(Some(root));

    assert_eq!(area_dir, format!("{}/area", root));
    assert_eq!(area_list, format!("{}/area/area.lst", root));
    assert_eq!(bug_file, format!("{}/area/bugs.txt", root));
    assert_eq!(idea_file, format!("{}/area/ideas.txt", root));
    assert_eq!(typo_file, format!("{}/area/typos.txt", root));
    assert_eq!(note_file, format!("{}/area/notes.txt", root));
    assert_eq!(shutdown_file, format!("{}/area/shutdown.txt", root));
    assert_eq!(copyover_file, format!("{}/area/copyover.txt", root));
    assert_eq!(mobprog_dir, format!("{}/area/MOBProgs", root));

    // Test configure_misc_dirs
    let (finger_dir, note_dir) = configure_misc_dirs_pub(Some(root));
    assert_eq!(finger_dir, format!("{}/finger/", root));
    assert_eq!(note_dir, format!("{}/notes/", root));

    // Verify trailing slashes
    assert!(finger_dir.ends_with('/'));
    assert!(note_dir.ends_with('/'));

    // Cleanup
    clear_chaos_env();
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_full_init_cycle_with_individual_overrides() {
    let _lock = INTEGRATION_MUTEX.lock().unwrap();
    clear_chaos_env();

    let tmp = env::temp_dir().join("chaos_integration_test_2");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(tmp.join("custom_player")).unwrap();
    fs::create_dir_all(tmp.join("custom_area")).unwrap();
    fs::create_dir_all(tmp.join("custom_finger")).unwrap();
    fs::create_dir_all(tmp.join("custom_notes")).unwrap();

    let player_path = tmp.join("custom_player").to_str().unwrap().to_string();
    let area_path = tmp.join("custom_area").to_str().unwrap().to_string();
    let finger_path = tmp.join("custom_finger").to_str().unwrap().to_string();
    let notes_path = tmp.join("custom_notes").to_str().unwrap().to_string();

    // Set individual overrides
    env::set_var("CHAOS_PLAYER_DIR", &player_path);
    env::set_var("CHAOS_AREA_DIR", &area_path);
    env::set_var("CHAOS_FINGER_DIR", &finger_path);
    env::set_var("CHAOS_NOTES_DIR", &notes_path);

    // Player dir should use the override
    let result = resolve_path("CHAOS_PLAYER_DIR", None, "player/", "../player/");
    assert_eq!(result, player_path);

    // Area dir should use the override
    let (area_dir, area_list, _, _, _, _, _, _, _) = configure_area_paths_pub(None);
    assert_eq!(area_dir, area_path);
    assert_eq!(area_list, format!("{}/area.lst", area_path));

    // Misc dirs should use overrides (with trailing slash enforcement)
    let (finger_dir, note_dir) = configure_misc_dirs_pub(None);
    assert_eq!(finger_dir, format!("{}/", finger_path));
    assert_eq!(note_dir, format!("{}/", notes_path));

    // Cleanup
    clear_chaos_env();
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_compiled_defaults_when_no_env_set() {
    let _lock = INTEGRATION_MUTEX.lock().unwrap();
    clear_chaos_env();

    // With no env vars and no root, should fall back to compiled defaults
    let player_dir = resolve_path("CHAOS_PLAYER_DIR", None, "player/", "../player/");
    assert_eq!(player_dir, "../player/");

    let player_temp = resolve_path("CHAOS_PLAYER_TEMP_DIR", None, "player/temp", "../player/temp");
    assert_eq!(player_temp, "../player/temp");
}

#[test]
fn test_path_exists_with_real_paths() {
    let _lock = INTEGRATION_MUTEX.lock().unwrap();

    let tmp = env::temp_dir().join("chaos_integration_test_3");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    assert!(path_exists(tmp.to_str().unwrap()));
    assert!(!path_exists(tmp.join("nonexistent").to_str().unwrap()));

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_join_path_various_combinations() {
    let _lock = INTEGRATION_MUTEX.lock().unwrap();

    assert_eq!(join_path("/env/dev/area", "area.lst"), "/env/dev/area/area.lst");
    assert_eq!(join_path("/env/dev/area/", "area.lst"), "/env/dev/area/area.lst");
    assert_eq!(join_path("area", "MOBProgs"), "area/MOBProgs");
    assert_eq!(join_path("", "fallback"), "fallback");
    assert_eq!(join_path("dir_only", ""), "dir_only");
}

#[test]
fn test_ensure_trailing_slash_various() {
    let _lock = INTEGRATION_MUTEX.lock().unwrap();

    let mut path1 = "/env/dev/player".to_string();
    ensure_trailing_slash(&mut path1);
    assert_eq!(path1, "/env/dev/player/");

    let mut path2 = "/env/dev/player/".to_string();
    ensure_trailing_slash(&mut path2);
    assert_eq!(path2, "/env/dev/player/");

    let mut path3 = String::new();
    ensure_trailing_slash(&mut path3);
    assert_eq!(path3, "");
}

#[test]
fn test_area_file_path_str_integration() {
    let _lock = INTEGRATION_MUTEX.lock().unwrap();
    clear_chaos_env();

    // Direct join_path test for area file resolution
    let area_dir = "/env/dev/area";
    let result = join_path(area_dir, "midgaard.are");
    assert_eq!(result, "/env/dev/area/midgaard.are");

    let result2 = join_path(area_dir, "school.are");
    assert_eq!(result2, "/env/dev/area/school.are");
}

#[test]
fn test_root_with_trailing_slash() {
    let _lock = INTEGRATION_MUTEX.lock().unwrap();
    clear_chaos_env();

    // Root with trailing slash should not double up
    let result = resolve_path("NONEXISTENT_VAR_XYZ", Some("/env/dev/"), "player/", "../player/");
    assert_eq!(result, "/env/dev/player/");

    // Root without trailing slash should add one
    let result2 = resolve_path("NONEXISTENT_VAR_XYZ2", Some("/env/dev"), "player/", "../player/");
    assert_eq!(result2, "/env/dev/player/");
}
