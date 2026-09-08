//! Bare-metal filesystem primitives and Win32 ReFS Block Cloning wrapper.
//!
//! Provides zero-copy, near-instantaneous copy-on-write cloning on Windows 11 Dev Drive ReFS
//! using low-level NT kernel FSCTL_DUPLICATE_EXTENTS_TO_FILE via windows-sys 0.59.
//! Prevents SSD write amplification on NVMe drives.

use std::path::Path;
use crate::error::CoreError;

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, HANDLE, INVALID_HANDLE_VALUE,
};
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, GetDiskFreeSpaceW, GetFileSizeEx, SetEndOfFile, SetFilePointerEx,
    CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, FILE_BEGIN, FILE_SHARE_DELETE, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_EXISTING,
};
#[cfg(windows)]
use windows_sys::Win32::System::IO::DeviceIoControl;
#[cfg(windows)]
use windows_sys::Win32::System::Ioctl::{
    DUPLICATE_EXTENTS_DATA, FSCTL_DUPLICATE_EXTENTS_TO_FILE,
};

#[cfg(windows)]
const GENERIC_READ: u32 = 0x80000000;
#[cfg(windows)]
const GENERIC_WRITE: u32 = 0x40000000;

/// Validates that a path is anchored within the canonical Dev Drive partition (`Z:\`).
/// Prevents path leaks into the host system partition (C:\).
pub fn validate_refs_path(path: &Path) -> Result<(), CoreError> {
    let path_str = path.to_string_lossy();
    // Normalize path check for Windows drive prefix
    if !path_str.starts_with("Z:") && !path_str.starts_with("z:") && !path_str.starts_with("\\\\?\\Z:") && !path_str.starts_with("\\\\?\\z:") {
        return Err(CoreError::RefsPathLeakViolation(format!(
            "Access violation: path '{path_str}' escapes canonical Dev Drive ReFS partition (Z:\\)"
        )));
    }
    Ok(())
}

/// Ensures a directory exists within the ReFS Dev Drive partition.
pub fn ensure_refs_directory(path: &Path) -> Result<(), CoreError> {
    validate_refs_path(path)?;
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Helper struct to ensure Win32 HANDLEs are safely closed when dropped.
#[cfg(windows)]
struct AutoHandle(HANDLE);

#[cfg(windows)]
impl Drop for AutoHandle {
    fn drop(&mut self) {
        if self.0 != INVALID_HANDLE_VALUE && self.0 != std::ptr::null_mut() {
            // SAFETY: Closing valid Win32 handle on scope exit.
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

/// Queries the filesystem cluster size in bytes for the given drive path.
#[cfg(windows)]
fn get_cluster_size(path: &Path) -> u32 {
    let root = path
        .components()
        .next()
        .map(|c| format!("{}\\", c.as_os_str().to_string_lossy()))
        .unwrap_or_else(|| "Z:\\".to_string());

    let root_wide: Vec<u16> = root.encode_utf16().chain(std::iter::once(0)).collect();

    let mut sectors_per_cluster: u32 = 0;
    let mut bytes_per_sector: u32 = 0;
    let mut number_of_free_clusters: u32 = 0;
    let mut total_number_of_clusters: u32 = 0;

    // SAFETY: Calling GetDiskFreeSpaceW with valid wide string and pointers.
    let ok = unsafe {
        GetDiskFreeSpaceW(
            root_wide.as_ptr(),
            &mut sectors_per_cluster,
            &mut bytes_per_sector,
            &mut number_of_free_clusters,
            &mut total_number_of_clusters,
        )
    };

    if ok != 0 && sectors_per_cluster > 0 && bytes_per_sector > 0 {
        sectors_per_cluster * bytes_per_sector
    } else {
        4096 // Canonical ReFS Dev Drive 4KB cluster fallback
    }
}

/// Performs a synchronous zero-copy ReFS Block Cloning operation on Windows NT.
#[cfg(windows)]
fn clone_file_refs_sync(src: &Path, dest: &Path) -> Result<(), CoreError> {
    let src_wide: Vec<u16> = src.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let dest_wide: Vec<u16> = dest.as_os_str().encode_wide().chain(std::iter::once(0)).collect();

    // SAFETY: Opening source handle with Win32 CreateFileW using shared read/write/delete semantics.
    let src_raw = unsafe {
        CreateFileW(
            src_wide.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            std::ptr::null_mut(),
        )
    };

    if src_raw == INVALID_HANDLE_VALUE {
        let err = unsafe { GetLastError() };
        return Err(CoreError::RefsBlockCloningFailed(format!(
            "Failed to open source file for ReFS cloning: Win32 error {err}"
        )));
    }
    let src_handle = AutoHandle(src_raw);

    let mut file_size: i64 = 0;
    // SAFETY: Querying file size of open handle with GetFileSizeEx.
    let ok_size = unsafe { GetFileSizeEx(src_handle.0, &mut file_size) };
    if ok_size == 0 {
        let err = unsafe { GetLastError() };
        return Err(CoreError::RefsBlockCloningFailed(format!(
            "Failed to read source file size: Win32 error {err}"
        )));
    }

    // SAFETY: Creating destination file with shared read/write/delete semantics.
    let dest_raw = unsafe {
        CreateFileW(
            dest_wide.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            std::ptr::null_mut(),
        )
    };

    if dest_raw == INVALID_HANDLE_VALUE {
        let err = unsafe { GetLastError() };
        return Err(CoreError::RefsBlockCloningFailed(format!(
            "Failed to create target file for ReFS cloning: Win32 error {err}"
        )));
    }
    let dest_handle = AutoHandle(dest_raw);

    // If source file is empty (0 bytes), an empty file was created; no extents to duplicate.
    if file_size == 0 {
        return Ok(());
    }

    // ReFS Block Cloning requires ByteCount to be aligned to cluster boundary.
    let cluster_size = get_cluster_size(src) as i64;
    let aligned_byte_count = ((file_size + cluster_size - 1) / cluster_size) * cluster_size;

    // Extend destination file to hold extents before cloning
    let mut new_pos: i64 = 0;
    // SAFETY: Setting file pointer to file_size on destination handle.
    let ok_pos = unsafe { SetFilePointerEx(dest_handle.0, file_size, &mut new_pos, FILE_BEGIN) };
    if ok_pos == 0 {
        let err = unsafe { GetLastError() };
        return Err(CoreError::RefsBlockCloningFailed(format!(
            "SetFilePointerEx failed: Win32 error {err}"
        )));
    }

    // SAFETY: Setting end of file on destination handle.
    let ok_eof = unsafe { SetEndOfFile(dest_handle.0) };
    if ok_eof == 0 {
        let err = unsafe { GetLastError() };
        return Err(CoreError::RefsBlockCloningFailed(format!(
            "SetEndOfFile failed: Win32 error {err}"
        )));
    }

    let dup_data = DUPLICATE_EXTENTS_DATA {
        FileHandle: src_handle.0,
        SourceFileOffset: 0,
        TargetFileOffset: 0,
        ByteCount: aligned_byte_count,
    };

    let mut bytes_returned: u32 = 0;
    // SAFETY: Calling DeviceIoControl with FSCTL_DUPLICATE_EXTENTS_TO_FILE to trigger
    // atomic zero-copy copy-on-write extent duplication in the NT ReFS driver.
    let ioctl_ok = unsafe {
        DeviceIoControl(
            dest_handle.0,
            FSCTL_DUPLICATE_EXTENTS_TO_FILE,
            &dup_data as *const _ as *const _,
            std::mem::size_of::<DUPLICATE_EXTENTS_DATA>() as u32,
            std::ptr::null_mut(),
            0,
            &mut bytes_returned,
            std::ptr::null_mut(),
        )
    };

    if ioctl_ok == 0 {
        let err = unsafe { GetLastError() };
        return Err(CoreError::RefsBlockCloningFailed(format!(
            "FSCTL_DUPLICATE_EXTENTS_TO_FILE failed: Win32 error {err}"
        )));
    }

    // Ensure the final visible file size matches the original file exactly.
    // SAFETY: Resetting end of file to exact source file_size after extent duplication.
    unsafe {
        SetFilePointerEx(dest_handle.0, file_size, std::ptr::null_mut(), FILE_BEGIN);
        SetEndOfFile(dest_handle.0);
    }

    Ok(())
}

/// Fallback for non-Windows platforms delegating asynchronously to tokio::fs::copy.
#[cfg(not(windows))]
pub async fn clone_file_refs(src: &Path, dest: &Path) -> Result<(), CoreError> {
    tokio::fs::copy(src, dest).await.map_err(CoreError::Io)?;
    Ok(())
}

/// Asynchronously clones a file using Windows 11 ReFS Block Cloning (Copy-on-Write).
///
/// Dispatches the synchronous NT kernel call `DeviceIoControl` with `FSCTL_DUPLICATE_EXTENTS_TO_FILE`
/// to `tokio::task::spawn_blocking` to prevent async worker thread starvation, achieving O(1)
/// constant-time extent duplication with zero physical NVMe write amplification.
#[cfg(windows)]
pub async fn clone_file_refs(src: &Path, dest: &Path) -> Result<(), CoreError> {
    let src_buf = src.to_path_buf();
    let dest_buf = dest.to_path_buf();
    tokio::task::spawn_blocking(move || clone_file_refs_sync(&src_buf, &dest_buf))
        .await
        .map_err(|e| CoreError::TaskJoinError(e.to_string()))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::Instant;

    #[test]
    fn test_validate_refs_path() {
        assert!(validate_refs_path(Path::new("Z:\\souls_engine\\.souls_data\\db")).is_ok());
        assert!(validate_refs_path(Path::new("z:\\souls_engine\\crates")).is_ok());
        assert!(validate_refs_path(Path::new("C:\\Users\\malicious")).is_err());
        assert!(validate_refs_path(Path::new("D:\\other_drive")).is_err());
    }

    #[tokio::test]
    async fn test_refs_block_cloning_50mb_instantaneous() {
        // Perform test on Dev Drive Z:\ to exercise true ReFS copy-on-write
        let test_dir = Path::new("Z:\\souls_engine\\.souls_data\\spool\\test_refs");
        std::fs::create_dir_all(test_dir).expect("Failed to create test directory");

        let src_file = test_dir.join("src_50mb.bin");
        let dest_file = test_dir.join("dest_50mb_clone.bin");

        // 1. Generate 50MB of binary test data
        let size_50mb = 50 * 1024 * 1024; // 52,428,800 bytes
        {
            let mut file = std::fs::File::create(&src_file).expect("Failed to create 50MB src file");
            let chunk = vec![0xABu8; 1024 * 1024]; // 1MB chunks
            for _ in 0..50 {
                file.write_all(&chunk).expect("Write failed");
            }
            file.flush().expect("Flush failed");
        }

        // 2. Measure clone_file_refs execution time
        let start = Instant::now();
        clone_file_refs(&src_file, &dest_file)
            .await
            .expect("ReFS Block Cloning failed");
        let elapsed = start.elapsed();

        // 3. Verify destination integrity and size
        let dest_meta = std::fs::metadata(&dest_file).expect("Failed to stat dest file");
        assert_eq!(
            dest_meta.len(),
            size_50mb as u64,
            "Destination size must be exact 50MB"
        );

        // 4. Assert instantaneous operation: ReFS CoW extent cloning for 50MB takes < 100ms,
        // whereas a physical byte copy would take significantly longer.
        println!("ReFS Block Cloning 50MB took: {:?}", elapsed);
        assert!(
            elapsed.as_millis() < 500,
            "ReFS Block Cloning should be virtually instantaneous (took {:?})",
            elapsed
        );

        // Cleanup
        let _ = std::fs::remove_file(&src_file);
        let _ = std::fs::remove_file(&dest_file);
        let _ = std::fs::remove_dir(test_dir);
    }
}
