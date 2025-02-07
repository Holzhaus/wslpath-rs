// Copyright (c) 2025 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Convert paths between WSL guest and Windows host.
//!
//! This library aims to offer functionality similar of the `wslpath` conversion tool [added in WSL
//! build 17046](https://learn.microsoft.com/en-us/windows/wsl/release-notes#wsl-34), but is
//! implemented in pure Rust.
//!
//! Existing crates such as [`wslpath`](https://crates.io/crates/wslpath) call `wsl.exe wslpath`
//! internally, which may lead to a lot of command invocations when multiple paths need to be
//! converted.

#![warn(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(rust_2018_idioms)]
#![deny(rust_2021_compatibility)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::all)]
#![deny(clippy::explicit_deref_methods)]
#![deny(clippy::explicit_into_iter_loop)]
#![deny(clippy::explicit_iter_loop)]
#![deny(clippy::must_use_candidate)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]

use typed_path::{
    Utf8UnixComponent, Utf8UnixPath, Utf8UnixPathBuf, Utf8WindowsComponent, Utf8WindowsPath,
    Utf8WindowsPathBuf, Utf8WindowsPrefix,
};

/// Represents an error that occurred during conversion.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// The input path is relative and thus cannot be converted.
    RelativePath,
    /// The input path prefix is invalid.
    InvalidPrefix,
}

/// Convert a Windows path to a WSL path.
///
/// The input path needs to be absolute. Path are normalized during conversion. UNC paths
/// (`\\?\C:\...`) are supported.
///
/// # Errors
///
/// If the path is not absolute, the method returns an [`Error::RelativePath`]. Paths not starting
/// with a drive letter will lead to an [`Error::InvalidPrefix`].
///
/// # Examples
///
/// ```
/// use wslpath_rs::{windows_to_wsl, Error};
///
/// // Regular absolute paths are supported
/// assert_eq!(windows_to_wsl("C:\\Windows").unwrap(), "/mnt/c/Windows");
/// assert_eq!(windows_to_wsl("D:\\foo\\..\\bar\\.\\baz.txt").unwrap(), "/mnt/d/bar/baz.txt");
/// assert_eq!(windows_to_wsl("C:\\Program Files (x86)\\Foo\\bar.txt").unwrap(), "/mnt/c/Program Files (x86)/Foo/bar.txt");
///
/// // UNC paths are supported
/// assert_eq!(windows_to_wsl("\\\\?\\C:\\Windows").unwrap(), "/mnt/c/Windows");
/// assert_eq!(windows_to_wsl("\\\\?\\D:\\foo\\..\\bar\\.\\baz.txt").unwrap(), "/mnt/d/bar/baz.txt");
/// assert_eq!(windows_to_wsl("\\\\?\\C:\\Program Files (x86)\\Foo\\bar.txt").unwrap(), "/mnt/c/Program Files (x86)/Foo/bar.txt");
///
/// // Relative paths are not supported
/// assert_eq!(windows_to_wsl("Program Files (x86)\\Foo\\bar.txt").unwrap_err(), Error::RelativePath);
/// assert_eq!(windows_to_wsl("..\\foo\\bar.txt").unwrap_err(), Error::RelativePath);
///
/// // Windows WSL paths are converted to the root
/// assert_eq!(windows_to_wsl("\\\\?\\UNC\\wsl.localhost\\distro\\home\\user\\file").unwrap(), "/home/user/file");
///
/// // Generic network paths are not supported right now
/// assert_eq!(windows_to_wsl("\\\\?\\UNC\\other.domain\\distro\\home\\user\\file").unwrap_err(), Error::InvalidPrefix);
/// ```
pub fn windows_to_wsl(windows_path: &str) -> Result<String, Error> {
    let path = Utf8WindowsPath::new(windows_path);
    if !path.is_absolute() {
        return Err(Error::RelativePath);
    }

    // "C:\foo" (6 chars) -> "/mnt/c/foo" (10 chars)
    let expected_length = windows_path.len() + 4;
    let mut output = Utf8UnixPathBuf::with_capacity(expected_length);
    for component in path.components() {
        match component {
            Utf8WindowsComponent::Prefix(prefix_component) => match prefix_component.kind() {
                Utf8WindowsPrefix::VerbatimDisk(disk) => {
                    output.push("/mnt");
                    output.push(disk.to_ascii_lowercase().to_string());
                }
                Utf8WindowsPrefix::Disk(disk) => {
                    output.push("/mnt");
                    output.push(disk.to_ascii_lowercase().to_string());
                }
                Utf8WindowsPrefix::VerbatimUNC(hostname, _) => {
                    // Assume that the path is inside the current wsl distro
                    if hostname == "wsl.localhost" {
                        output.push("/");
                    } else {
                        return Err(Error::InvalidPrefix);
                    }
                }
                _ => {
                    return Err(Error::InvalidPrefix);
                }
            },
            Utf8WindowsComponent::RootDir => (),
            Utf8WindowsComponent::CurDir => output.push("."),
            Utf8WindowsComponent::Normal(name) => output.push(name),
            Utf8WindowsComponent::ParentDir => output.push(".."),
        };
    }

    Ok(output.normalize().into_string())
}

/// Convert a WSL path to a Windows path.
///
/// The input path needs to be absolute. Path are normalized during conversion.
///
/// # Errors
///
/// If the path is not absolute, the method returns an [`Error::RelativePath`]. Paths not starting
/// with with `/mnt/<driveletter>` will lead to an [`Error::InvalidPrefix`].
///
/// # Examples
///
/// ```
/// use wslpath_rs::{wsl_to_windows, Error};
///
/// // Absolute paths are supported
/// assert_eq!(wsl_to_windows("/mnt/c/Windows").unwrap(), "C:\\Windows");
/// assert_eq!(wsl_to_windows("/mnt/d/foo/../bar/./baz.txt").unwrap(), "D:\\bar\\baz.txt");
/// assert_eq!(wsl_to_windows("/mnt/c/Program Files (x86)/Foo/bar.txt").unwrap(), "C:\\Program Files (x86)\\Foo\\bar.txt");
///
/// // Absolute paths not starting with `/mnt/<driveletter>` are not supported
/// assert_eq!(wsl_to_windows("/etc/fstab").unwrap_err(), Error::InvalidPrefix);
/// assert_eq!(wsl_to_windows("/mnt/my_custom_mount/foo/bar.txt").unwrap_err(), Error::InvalidPrefix);
///
/// // Relative paths are not supported
/// assert_eq!(wsl_to_windows("Program Files (x86)/Foo/bar.txt").unwrap_err(), Error::RelativePath);
/// assert_eq!(wsl_to_windows("../foo/bar.txt").unwrap_err(), Error::RelativePath);
/// ```
pub fn wsl_to_windows(wsl_path: &str) -> Result<String, Error> {
    let path = Utf8UnixPath::new(wsl_path);
    if !path.is_absolute() {
        return Err(Error::RelativePath);
    }

    let mut components = path.components();
    if components.next() != Some(Utf8UnixComponent::RootDir) {
        return Err(Error::InvalidPrefix);
    }
    if components.next() != Some(Utf8UnixComponent::Normal("mnt")) {
        return Err(Error::InvalidPrefix);
    }

    // "/mnt/c/foo" (10 chars) -> "C:\foo" (6 chars)
    let expected_length = wsl_path.len();
    let mut output = Utf8WindowsPathBuf::with_capacity(expected_length);
    if let Some(Utf8UnixComponent::Normal(drive)) = components.next() {
        if drive.len() != 1 {
            return Err(Error::InvalidPrefix);
        }

        output.push(format!("{}:\\", drive.to_ascii_uppercase()));
    } else {
        return Err(Error::InvalidPrefix);
    }

    for component in components {
        match component {
            Utf8UnixComponent::RootDir => (),
            Utf8UnixComponent::CurDir => output.push("."),
            Utf8UnixComponent::Normal(name) => output.push(name),
            Utf8UnixComponent::ParentDir => output.push(".."),
        };
    }

    Ok(output.normalize().into_string())
}
