use crate::inter::helpers;
use crate::inter::types;

use types::ReparseDataBuffer;
use types::{MOUNT_POINT_REPARSE_BUFFER_HEADER_SIZE, REPARSE_DATA_BUFFER_HEADER_SIZE};

use std::cmp;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::os::windows::ffi::OsStringExt;
use std::os::windows::io::AsRawHandle;
use std::path::{Path, PathBuf};
use std::ptr;
use std::slice;

use winapi::um::winnt::{IO_REPARSE_TAG_MOUNT_POINT, MAXIMUM_REPARSE_DATA_BUFFER_SIZE};

/// This prefix indicates to NTFS that the path is to be treated as a non-interpreted
/// path in the virtual file system.
const NON_INTERPRETED_PATH_PREFIX: [u16; 4] = [b'\\' as u16, b'?' as _, b'?' as _, b'\\' as _];
const WCHAR_SIZE: u16 = std::mem::size_of::<u16>() as _;

pub fn create(target: &Path, junction: &Path) -> io::Result<()> {
    const UNICODE_NULL_SIZE: u16 = WCHAR_SIZE;
    const MAX_AVAILABLE_PATH_BUFFER: u16 = MAXIMUM_REPARSE_DATA_BUFFER_SIZE as u16
        - REPARSE_DATA_BUFFER_HEADER_SIZE
        - MOUNT_POINT_REPARSE_BUFFER_HEADER_SIZE
        - 2 * UNICODE_NULL_SIZE;

    // We're using low-level APIs to create the junction, and these are more picky about paths.
    // For example, forward slashes cannot be used as a path separator, so we should try to
    // canonicalize the path first.
    let mut target = helpers::get_full_path(target)?;
    fs::create_dir(junction)?;
    let file = helpers::open_reparse_point(junction, true)?;
    // "\??\" + target
    let len = NON_INTERPRETED_PATH_PREFIX
        .len()
        .saturating_add(target.len());
    // Len without `UNICODE_NULL` at the end
    let target_len_in_bytes =
        (cmp::min(len, std::u16::MAX as usize) as u16).saturating_mul(WCHAR_SIZE);
    // Check if `target_wchar.len()` may lead to a buffer overflow.
    if target_len_in_bytes > MAX_AVAILABLE_PATH_BUFFER {
        return Err(io::Error::new(io::ErrorKind::Other, "`target` is too long"));
    }

    let mut target_wchar: Vec<u16> = Vec::with_capacity(len);
    target_wchar.extend(&NON_INTERPRETED_PATH_PREFIX);
    target_wchar.append(&mut target);

    // Redefine the above char array into a ReparseDataBuffer we can work with
    let mut data = vec![0; MAXIMUM_REPARSE_DATA_BUFFER_SIZE as usize];
    #[warn(clippy::cast_ptr_alignment)]
    let rdb = data.as_mut_ptr().cast::<ReparseDataBuffer>();
    let in_buffer_size: u16 = unsafe {
        // Set the type of reparse point we are creating
        (*rdb).reparse_tag = IO_REPARSE_TAG_MOUNT_POINT;
        (*rdb).reserved = 0;

        // Copy the junction's target
        (*rdb).reparse_buffer.substitute_name_offset = 0;
        (*rdb).reparse_buffer.substitute_name_length = target_len_in_bytes;

        // Copy the junction's link name
        (*rdb).reparse_buffer.print_name_offset = target_len_in_bytes + UNICODE_NULL_SIZE;
        (*rdb).reparse_buffer.print_name_length = 0;

        // Safe because we checked `MAX_AVAILABLE_PATH_BUFFER`
        ptr::copy_nonoverlapping(
            target_wchar.as_ptr().cast::<u16>(),
            (*rdb).reparse_buffer.path_buffer.as_mut_ptr(),
            target_wchar.len(),
        );

        // Set the total size of the data buffer
        (*rdb).reparse_data_length = target_len_in_bytes
            .wrapping_add(MOUNT_POINT_REPARSE_BUFFER_HEADER_SIZE + 2 * UNICODE_NULL_SIZE);
        (*rdb)
            .reparse_data_length
            .wrapping_add(REPARSE_DATA_BUFFER_HEADER_SIZE)
    };

    helpers::set_reparse_point(file.as_raw_handle(), rdb, u32::from(in_buffer_size))
}

pub fn delete(junction: &Path) -> io::Result<()> {
    let file = helpers::open_reparse_point(junction, true)?;
    helpers::delete_reparse_point(file.as_raw_handle())
}

pub fn exists(junction: &Path) -> io::Result<bool> {
    if !junction.exists() {
        return Ok(false);
    }
    let file = helpers::open_reparse_point(junction, false)?;
    // Allocate enough space to fit the maximum sized reparse data buffer
    let mut data = vec![0; MAXIMUM_REPARSE_DATA_BUFFER_SIZE as usize];
    let rdb = helpers::get_reparse_data_point(file.as_raw_handle(), &mut data)?;
    // The reparse tag indicates if this is a junction or not
    Ok(rdb.reparse_tag == IO_REPARSE_TAG_MOUNT_POINT)
}

pub fn get_target(junction: &Path) -> io::Result<PathBuf> {
    if !junction.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "`junction` does not exist",
        ));
    }
    let file = helpers::open_reparse_point(junction, false)?;
    let mut data = vec![0; MAXIMUM_REPARSE_DATA_BUFFER_SIZE as usize];
    let rdb = helpers::get_reparse_data_point(file.as_raw_handle(), &mut data)?;
    if rdb.reparse_tag == IO_REPARSE_TAG_MOUNT_POINT {
        let offset = rdb.reparse_buffer.substitute_name_offset / WCHAR_SIZE;
        let len = rdb.reparse_buffer.substitute_name_length / WCHAR_SIZE;
        let mut wide = unsafe {
            let buf = rdb.reparse_buffer.path_buffer.as_ptr().add(offset as usize);
            slice::from_raw_parts(buf, len as usize)
        };
        // In case of "\??\C:\foo\bar"
        if wide.starts_with(&NON_INTERPRETED_PATH_PREFIX) {
            wide = &wide[(NON_INTERPRETED_PATH_PREFIX.len())..];
        }
        Ok(PathBuf::from(OsString::from_wide(wide)))
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "not a reparse tag mount point",
        ))
    }
}
