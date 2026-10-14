// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Verified load of the pinned ONNX Runtime library, with the telemetry setting of the environment as a parameter.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::variant::TelemetryEvents;

/// The environment variable that switches off the 1DS telemetry of ONNX Runtime for the process lifetime.
pub const DISABLE_TELEMETRY_VARIABLE: &str = "ORT_DISABLE_TELEMETRY";

/// One pinned ONNX Runtime library of an official Microsoft release asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinnedRuntime {
    /// File name of the library inside the installation directory.
    pub file_name: &'static str,
    /// Lower-case hex SHA-256 of the library file.
    pub sha256: &'static str,
}

/// `onnxruntime.dll` from `onnxruntime-win-x64-1.31.0.zip`.
pub const WINDOWS_X64: PinnedRuntime = PinnedRuntime {
    file_name: "onnxruntime.dll",
    sha256: "0c42dc4e278f0968fdf9ebf1b1e54a74c657a9942e70803e453c91993451cf4c",
};

/// `libonnxruntime.so.1.31.0` from `onnxruntime-linux-x64-1.31.0.tgz`.
pub const LINUX_X64: PinnedRuntime = PinnedRuntime {
    file_name: "libonnxruntime.so.1.31.0",
    sha256: "469849977cb60744ea160927e1a3a0643fd4c55c0aff743251c6d293402e334e",
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

/// Sets `ORT_DISABLE_TELEMETRY=1` in the process environment.
///
/// # Safety
///
/// No other thread may read or write the process environment while this runs; call it first in `main`.
pub unsafe fn disable_telemetry_by_environment() {
    unsafe { std::env::set_var(DISABLE_TELEMETRY_VARIABLE, "1") };
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
    /// The ONNX Runtime environment could not be created.
    Environment(ort::Error),
    /// `ort` already holds a runtime that the process mapped from another path.
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
            } => write!(
                f,
                "`{}` has SHA-256 {actual}, expected {expected}",
                path.display()
            ),
            Self::SearchPath(source) => write!(f, "cannot restrict the DLL search path: {source}"),
            Self::Ort(source) => write!(f, "{source}"),
            Self::Environment(source) => {
                write!(f, "cannot create the ONNX Runtime environment: {source}")
            }
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

/// The verified runtime: the file it was loaded from and the one environment of the process.
#[derive(Debug)]
pub struct LoadedRuntime {
    /// The absolute path of the loaded library.
    pub path: PathBuf,
    /// The ONNX Runtime environment.
    pub environment: ort::environment::Environment,
}

/// Verifies the pinned library in `dir`, loads it by absolute path into `ort` and creates the environment with `events`.
///
/// # Errors
///
/// [`LoadError::NotAbsolute`], [`LoadError::Read`] and [`LoadError::HashMismatch`] before anything is loaded;
/// [`LoadError::SearchPath`] and [`LoadError::Ort`] when the verified file cannot be loaded;
/// [`LoadError::LoadedElsewhere`] when `ort` already holds a runtime from another path;
/// [`LoadError::Environment`] when the environment cannot be created.
pub fn load(
    dir: &Path,
    pinned: &PinnedRuntime,
    events: TelemetryEvents,
) -> Result<LoadedRuntime, LoadError> {
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
    let builder = ort::init_from(&path).map_err(LoadError::Ort)?;
    let mapped = platform::loaded_module_path(pinned.file_name);
    if !same_file(mapped.as_deref(), &path) {
        return Err(LoadError::LoadedElsewhere {
            verified: path,
            mapped,
        });
    }
    let builder = match events {
        TelemetryEvents::OrtDefault => builder,
        TelemetryEvents::Off => builder.with_telemetry(false),
    };
    let environment = builder.build().map_err(LoadError::Environment)?;
    Ok(LoadedRuntime { path, environment })
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
