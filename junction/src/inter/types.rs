use winapi::shared::guiddef;

/// Reparse Data Buffer header size = `sizeof(u32) + 2 * sizeof(u16)`
pub const REPARSE_DATA_BUFFER_HEADER_SIZE: u16 = 8;
/// Reparse GUID Data Buffer header size = `sizeof(u32) + 2*sizeof(u16) + sizeof(GUID)`
pub const REPARSE_GUID_DATA_BUFFER_HEADER_SIZE: u16 = 24;
/// MountPointReparseBuffer header size = `4 * sizeof(u16)`
pub const MOUNT_POINT_REPARSE_BUFFER_HEADER_SIZE: u16 = 8;

#[repr(C)]
#[derive(Debug)]
pub struct MountPointReparseBuffer {
    /// Offset, in bytes, of the substitute name string in the `path_buffer` array.
    /// Note that this offset must be divided by `sizeof(u16)` to get the array index.
    pub substitute_name_offset: u16,
    /// Length, in bytes, of the substitute name string. If this string is `NULL`-terminated,
    /// it does not include space for the `UNICODE_NULL` character.
    pub substitute_name_length: u16,
    /// Offset, in bytes, of the print name string in the `path_buffer` array.
    /// Note that this offset must be divided by `sizeof(u16)` to get the array index.
    pub print_name_offset: u16,
    /// Length, in bytes, of the print name string. If this string is `NULL`-terminated,
    /// it does not include space for the `UNICODE_NULL` character.
    pub print_name_length: u16,
    /// A buffer containing the Unicode-encoded path string. The path string contains the
    /// substitute name string and print name string. The substitute name and print name strings
    /// can appear in any order in the path_buffer. (To locate the substitute name and print name
    /// strings in the path_buffer, use the `substitute_name_offset`, `substitute_name_length`,
    /// `print_name_offset`, and `print_name_length` members.)
    pub path_buffer: [u16; 1],
}

/// This structure contains reparse point data for a Microsoft reparse point.
///
/// Read more:
/// * https://msdn.microsoft.com/en-us/windows/desktop/ff552012
/// * https://www.pinvoke.net/default.aspx/Structures.REPARSE_DATA_BUFFER
#[repr(C)]
#[derive(Debug)]
pub struct ReparseDataBuffer {
    /// Reparse point tag. Must be a Microsoft reparse point tag.
    pub reparse_tag: u32,
    /// Size, in bytes, of the reparse data in the `data_buffer` member.
    /// Or the size of the `path_buffer` field, in bytes, plus 8 (= 4 * sizeof(u16))
    pub reparse_data_length: u16,
    /// Reversed. It SHOULD be set to 0, and MUST be ignored.
    pub reserved: u16,
    pub reparse_buffer: MountPointReparseBuffer,
}

#[repr(C)]
#[derive(Debug)]
pub struct GenericReparseBuffer {
    /// Microsoft-defined data for the reparse point.
    pub data_buffer: [u8; 1],
}

/// Used by all third-party layered drivers to store data for a reparse point.
///
/// Each reparse point contains one instance of a `ReparseGuidDataBuffer` structure.
///
/// Read more:
/// * https://docs.microsoft.com/en-us/windows/desktop/api/winnt/ns-winnt-_reparse_guid_data_buffer
#[repr(C)]
pub struct ReparseGuidDataBuffer {
    /// Reparse point tag. This member identifies the structure of the user-defined
    /// reparse data.
    pub reparse_tag: u32,
    /// The size of the reparse data in the `data_buffer` member, in bytes. This
    /// value may vary with different tags and may vary between two uses of the
    /// same tag.
    pub reparse_data_length: u16,
    /// Reserved; do not use.
    pub reserved: u16,
    /// A `GUID` that uniquely identifies the reparse point. When setting a reparse
    /// point, the application must provide a non-`NULL` `GUID` in the `reparse_guid`
    /// member. When retrieving a reparse point from the file system, `reparse_guid`
    /// is the `GUID` assigned when the reparse point was set.
    pub reparse_guid: guiddef::GUID,
    /// The user-defined data for the reparse point. The contents are determined by
    /// the reparse point implementer. The tag in the `reparse_tag` member and the
    /// `GUID` in the `reparse_guid` member indicate how the data is to be interpreted.
    pub generic: GenericReparseBuffer,
}

impl std::fmt::Debug for ReparseGuidDataBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReparseGuidDataBuffer")
            .field("reparse_tag", &self.reparse_tag)
            .field("reparse_data_length", &self.reparse_data_length)
            .field("reserved", &self.reserved)
            .field(
                "reparse_guid",
                &format_args!(
                    "{}:{}:{}:{:?}",
                    self.reparse_guid.Data1, self.reparse_guid.Data2, self.reparse_guid.Data3, self.reparse_guid.Data4,
                ),
            )
            .field("generic", &self.generic.data_buffer)
            .finish()
    }
}
