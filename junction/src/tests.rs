#![allow(dead_code)]
#![allow(unused_imports)]

use crate::{create, delete, exists, get_target};
use std::fs::File;
use std::io;
use std::io::Write;
use std::{fs, os::windows::fs::symlink_file};

// https://docs.microsoft.com/en-us/windows/desktop/debug/system-error-codes
const ERROR_NOT_A_REPARSE_POINT: i32 = 0x1126;
const ERROR_ALREADY_EXISTS: i32 = 0xb7;

#[allow(unused_macros)]
macro_rules! check {
    ($e:expr) => {
        match $e {
            Ok(t) => t,
            Err(e) => panic!("{} failed with: {}", stringify!($e), e),
        }
    };
}

fn create_tempdir() -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix("junction-test-")
        .tempdir_in("target/debug")
        .unwrap()
}

#[test]
fn create_dir_all_with_junctions() {
    let tmpdir = create_tempdir();
    let target = tmpdir.path().join("target");

    let junction = tmpdir.path().join("junction");
    let b = junction.join("a/b");

    fs::create_dir_all(&target).unwrap();

    check!(create(&target, &junction));
    check!(fs::create_dir_all(&b));
    // the junction itself is not a directory, but `is_dir()` on a Path
    // follows links
    assert!(junction.is_dir());
    assert!(b.exists());
}

#[test]
fn create_recursive_rmdir() {
    let tmpdir = create_tempdir();
    let d1 = tmpdir.path().join("d1"); // "d1"
    let dt = d1.join("t"); // "d1/t"
    let dtt = dt.join("t"); // "d1/t/t"
    let d2 = tmpdir.path().join("d2"); // "d2"
    let canary = d2.join("do_not_delete"); // "d2/do_not_delete"

    check!(fs::create_dir_all(&dtt));
    check!(fs::create_dir_all(&d2));
    check!(check!(File::create(&canary)).write_all(b"foo"));

    check!(create(&d2, &dt.join("d2"))); // "d1/t/d2" -> "d2"

    let _ = symlink_file(&canary, &d1.join("canary")); // d1/canary -> d2/do_not_delete
    check!(fs::remove_dir_all(&d1));

    assert!(!d1.is_dir());
    assert!(canary.exists());
}

#[test]
fn create_recursive_rmdir_of_symlink() {
    // test we do not recursively delete a symlink but only dirs.
    let tmpdir = create_tempdir();
    let link = tmpdir.path().join("link");
    let dir = tmpdir.path().join("dir");
    let canary = dir.join("do_not_delete");
    check!(fs::create_dir_all(&dir));
    check!(check!(File::create(&canary)).write_all(b"foo"));
    check!(create(&dir, &link));
    check!(fs::remove_dir_all(&link));

    assert!(!link.is_dir());
    assert!(canary.exists());
}

#[test]
fn create_directory_exist_before() {
    let tmpdir = create_tempdir();

    let target = tmpdir.path().join("target");
    let junction = tmpdir.path().join("junction");

    check!(fs::create_dir_all(&junction));

    match create(&target, &junction) {
        Err(ref e) if e.raw_os_error() == Some(ERROR_ALREADY_EXISTS) => {}
        _ => panic!("directory exists before creating"),
    }
}

#[test]
fn create_target_no_exist() {
    let tmpdir = create_tempdir();

    let target = tmpdir.path().join("target");
    let junction = tmpdir.path().join("junction");

    match create(&target, &junction) {
        Ok(()) => {}
        _ => panic!("junction should point to non exist target path"),
    }
}

#[test]
fn delete_junctions() {
    let tmpdir = create_tempdir();

    let non_existence_dir = tmpdir.path().join("non_existence_dir");
    match delete(&non_existence_dir) {
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {}
        _ => panic!("target path does not exist or is not a directory"),
    }

    let dir_not_junction = tmpdir.path().join("dir_not_junction");
    check!(fs::create_dir_all(&dir_not_junction));
    match delete(&dir_not_junction) {
        Err(ref e) if e.raw_os_error() == Some(ERROR_NOT_A_REPARSE_POINT) => {}
        _ => panic!("target path is not a junction point"),
    }

    let file = tmpdir.path().join("foo-file");
    check!(check!(File::create(&file)).write_all(b"foo"));
    match delete(&file) {
        Err(ref e) if e.raw_os_error() == Some(ERROR_NOT_A_REPARSE_POINT) => {}
        _ => panic!("target path is not a junction point"),
    }
}

#[test]
fn exists_verify() {
    let tmpdir = create_tempdir();

    // Check no such directory or file
    let no_such_dir = tmpdir.path().join("no_such_dir");
    assert_eq!(check!(exists(&no_such_dir)), false);

    // Target exists but not a junction
    let no_such_file = tmpdir.path().join("file");
    check!(check!(File::create(&no_such_file)).write_all(b"foo"));
    match exists(&no_such_file) {
        Err(ref e) if e.raw_os_error() == Some(ERROR_NOT_A_REPARSE_POINT) => {}
        _ => panic!("target exists but not a junction"),
    }

    let target = tmpdir.path().join("target");
    let junction = tmpdir.path().join("junction");
    let file = target.join("file");
    let junction_file = junction.join("file");

    check!(fs::create_dir_all(&target));
    check!(check!(File::create(&file)).write_all(b"foo"));

    assert!(
        !junction_file.exists(),
        "file should not be located until junction created"
    );
    assert_eq!(check!(exists(&junction)), false, "junction not created yet");

    check!(create(&target, &junction));
    assert_eq!(check!(exists(&junction)), true, "junction should exist now");
    assert_eq!(&check!(get_target(&junction)), &target);
    assert!(
        junction_file.exists(),
        "file should be accessible via the junction"
    );

    check!(delete(&junction));
    match exists(&junction) {
        Err(ref e) if e.raw_os_error() == Some(ERROR_NOT_A_REPARSE_POINT) => {}
        _ => panic!("junction had been deleted"),
    }
    assert!(
        !junction_file.exists(),
        "file should not be located after junction deleted"
    );
    assert!(junction.exists(), "directory should not be deleted");
}

#[test]
fn get_target_user_dirs() {
    // junction
    assert_eq!(
        check!(get_target(r"C:\Users\Default User")).to_str(),
        Some(r"C:\Users\Default"),
    );
    // junction with special permissions
    assert_eq!(
        check!(get_target(r"C:\Documents and Settings\")).to_str(),
        Some(r"C:\Users"),
    );

    let tmpdir = create_tempdir();

    let non_existence_dir = tmpdir.path().join("non_existence_dir");
    match get_target(&non_existence_dir) {
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {}
        _ => panic!("target path does not exist or is not a directory"),
    }

    let dir_not_junction = tmpdir.path().join("dir_not_junction");
    check!(fs::create_dir_all(&dir_not_junction));
    match get_target(&dir_not_junction) {
        Err(ref e) if e.raw_os_error() == Some(ERROR_NOT_A_REPARSE_POINT) => {}
        _ => panic!("target path is not a junction point"),
    }

    let file = tmpdir.path().join("foo-file");
    check!(check!(File::create(&file)).write_all(b"foo"));
    match get_target(&file) {
        Err(ref e) if e.raw_os_error() == Some(ERROR_NOT_A_REPARSE_POINT) => {}
        _ => panic!("target path is not a junction point"),
    }
}
