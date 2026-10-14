// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Spike for O-108: load the official ONNX Runtime library with `ort`'s `load-dynamic` from an absolute path after a SHA-256 check.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

/// One pinned ONNX Runtime library of an official Microsoft release asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinnedRuntime {
    /// File name of the library inside the installation directory.
    pub file_name: &'static str,
    /// Lower-case hex SHA-256 of the library file.
    pub sha256: &'static str,
    /// Abbreviated commit of the release tag, as the library reports it in its build information.
    pub git_commit: &'static str,
}

/// `onnxruntime.dll` from `onnxruntime-win-x64-1.28.3.zip`.
pub const WINDOWS_X64: PinnedRuntime = PinnedRuntime {
    file_name: "onnxruntime.dll",
    sha256: "4d2774a5f64e4a230b16b74b167171e92f65b0683c19680c1d04fb284b92a6e2",
    git_commit: "0d68ff6",
};

/// `libonnxruntime.so.1.28.3` from `onnxruntime-linux-x64-1.28.3.tgz`.
pub const LINUX_X64: PinnedRuntime = PinnedRuntime {
    file_name: "libonnxruntime.so.1.28.3",
    sha256: "5a1ce74e56e8c4b278d5a9e9d12b7192d9f70811084dde9616da4fa1710efc5a",
    git_commit: "0d68ff6",
};

/// Returns the pinned library for the compilation target, or `None` for a target that ships none.
#[must_use]
pub fn pinned_for_target() -> Option<&'static PinnedRuntime> {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        Some(&WINDOWS_X64)
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Some(&LINUX_X64)
    } else {
        None
    }
}

/// Why the runtime was not loaded.
#[derive(Debug)]
pub enum LoadError {
    /// The directory is not an absolute path.
    NotAbsolute(PathBuf),
    /// The library file could not be read.
    Read {
        /// The file that was read.
        path: PathBuf,
        /// The I/O failure.
        source: io::Error,
    },
    /// The library file does not have the pinned SHA-256; it was not loaded.
    HashMismatch {
        /// The file that was hashed.
        path: PathBuf,
        /// The pinned hash.
        expected: &'static str,
        /// The hash of the file.
        actual: String,
    },
    /// The process-wide DLL search path could not be restricted.
    SearchPath(io::Error),
    /// `ort` could not load the verified library.
    Ort(ort::LoadDynamicError),
    /// `ort` already holds a runtime that the process mapped from another path; the verified file was not used.
    LoadedElsewhere {
        /// The verified file.
        verified: PathBuf,
        /// The path the process mapped the runtime from, if it could be read.
        mapped: Option<PathBuf>,
    },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAbsolute(dir) => write!(f, "`{}` is not an absolute path", dir.display()),
            Self::Read { path, source } => write!(f, "cannot read `{}`: {source}", path.display()),
            Self::HashMismatch {
                path,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "`{}` has SHA-256 {actual}, expected {expected}",
                    path.display()
                )
            }
            Self::SearchPath(source) => write!(f, "cannot restrict the DLL search path: {source}"),
            Self::Ort(source) => write!(f, "{source}"),
            Self::LoadedElsewhere { verified, mapped } => match mapped {
                Some(mapped) => write!(
                    f,
                    "the runtime was loaded from `{}` instead of `{}`",
                    mapped.display(),
                    verified.display()
                ),
                None => write!(f, "the runtime is not mapped from `{}`", verified.display()),
            },
        }
    }
}

impl std::error::Error for LoadError {}

/// Verifies the pinned library in `dir` and loads it by absolute path into `ort`.
///
/// Returns the path of the loaded file.
///
/// # Errors
///
/// [`LoadError::NotAbsolute`], [`LoadError::Read`] and [`LoadError::HashMismatch`] before anything is loaded;
/// [`LoadError::SearchPath`] and [`LoadError::Ort`] when the verified file cannot be loaded.
pub fn load(dir: &Path, pinned: &PinnedRuntime) -> Result<PathBuf, LoadError> {
    if !dir.is_absolute() {
        return Err(LoadError::NotAbsolute(dir.to_path_buf()));
    }
    let path = dir.join(pinned.file_name);
    let bytes = fs::read(&path).map_err(|source| LoadError::Read {
        path: path.clone(),
        source,
    })?;
    let actual = hex(&Sha256::digest(&bytes));
    if actual != pinned.sha256 {
        return Err(LoadError::HashMismatch {
            path,
            expected: pinned.sha256,
            actual,
        });
    }
    platform::restrict_library_search().map_err(LoadError::SearchPath)?;
    let _ = ort::init_from(&path).map_err(LoadError::Ort)?.commit();
    let mapped = platform::loaded_module_path(pinned.file_name);
    if !same_file(mapped.as_deref(), &path) {
        return Err(LoadError::LoadedElsewhere {
            verified: path,
            mapped,
        });
    }
    Ok(path)
}

/// Returns the path from which the process mapped the pinned library, or `None` when it is not loaded.
#[must_use]
pub fn loaded_runtime_path(pinned: &PinnedRuntime) -> Option<PathBuf> {
    platform::loaded_module_path(pinned.file_name)
}

fn same_file(mapped: Option<&Path>, verified: &Path) -> bool {
    match (mapped.map(fs::canonicalize), fs::canonicalize(verified)) {
        (Some(Ok(mapped)), Ok(verified)) => mapped == verified,
        (Some(Ok(_) | Err(_)) | None, Ok(_) | Err(_)) => false,
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Returns an ONNX model with one `Identity` node from float input `x` of shape `[3]` to output `y`.
#[must_use]
pub fn identity_model() -> Vec<u8> {
    let tensor_type = |name: &str| {
        let dim = field_bytes(1, &varint_field(1, 3));
        let shape = field_bytes(2, &dim);
        let tensor = [varint_field(1, 1), shape].concat();
        let type_proto = field_bytes(1, &tensor);
        [string_field(1, name), field_bytes(2, &type_proto)].concat()
    };
    let node = [
        string_field(1, "x"),
        string_field(2, "y"),
        string_field(3, "identity"),
        string_field(4, "Identity"),
    ]
    .concat();
    let graph = [
        field_bytes(1, &node),
        string_field(2, "spike"),
        field_bytes(11, &tensor_type("x")),
        field_bytes(12, &tensor_type("y")),
    ]
    .concat();
    let opset = [string_field(1, ""), varint_field(2, 13)].concat();
    [
        varint_field(1, 7),
        field_bytes(7, &graph),
        field_bytes(8, &opset),
    ]
    .concat()
}

fn varint(mut value: u64) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let low = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(low);
            return out;
        }
        out.push(low | 0x80);
    }
}

fn varint_field(number: u64, value: u64) -> Vec<u8> {
    [varint(number << 3), varint(value)].concat()
}

fn field_bytes(number: u64, bytes: &[u8]) -> Vec<u8> {
    [
        varint((number << 3) | 2),
        varint(bytes.len() as u64),
        bytes.to_vec(),
    ]
    .concat()
}

fn string_field(number: u64, value: &str) -> Vec<u8> {
    field_bytes(number, value.as_bytes())
}

#[cfg(windows)]
mod platform {
    use std::io;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::path::PathBuf;

    const LOAD_LIBRARY_SEARCH_DEFAULT_DIRS: u32 = 0x0000_1000;
    const MAX_PATH_WIDE: usize = 32_768;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetDefaultDllDirectories(flags: u32) -> i32;
        fn GetModuleHandleW(name: *const u16) -> *mut core::ffi::c_void;
        fn GetModuleFileNameW(module: *mut core::ffi::c_void, buffer: *mut u16, size: u32) -> u32;
    }

    pub(crate) fn restrict_library_search() -> io::Result<()> {
        let ok = unsafe { SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_DEFAULT_DIRS) };
        if ok == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub(crate) fn loaded_module_path(file_name: &str) -> Option<PathBuf> {
        let wide: Vec<u16> = std::ffi::OsStr::new(file_name)
            .encode_wide()
            .chain(Some(0))
            .collect();
        let module = unsafe { GetModuleHandleW(wide.as_ptr()) };
        if module.is_null() {
            return None;
        }
        let mut buffer = vec![0_u16; MAX_PATH_WIDE];
        let length =
            unsafe { GetModuleFileNameW(module, buffer.as_mut_ptr(), MAX_PATH_WIDE as u32) };
        let filled = buffer.get(..length as usize)?;
        Some(PathBuf::from(std::ffi::OsString::from_wide(filled)))
    }
}

#[cfg(not(windows))]
mod platform {
    use std::io;
    use std::path::PathBuf;

    pub(crate) fn restrict_library_search() -> io::Result<()> {
        Ok(())
    }

    pub(crate) fn loaded_module_path(file_name: &str) -> Option<PathBuf> {
        let maps = std::fs::read_to_string("/proc/self/maps").ok()?;
        maps.lines()
            .filter_map(|line| line.split_whitespace().nth(5))
            .find(|path| path.rsplit('/').next() == Some(file_name))
            .map(PathBuf::from)
    }
}
