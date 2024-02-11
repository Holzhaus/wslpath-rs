// Copyright (c) 2024 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use typed_path::{Utf8UnixPathBuf, Utf8WindowsComponent, Utf8WindowsPath, Utf8WindowsPrefix};

#[derive(Debug)]
pub enum Error {
    RelativePathError,
    InvalidPrefixError,
}

pub fn windows_to_wsl(windows_path: &str) -> Result<String, Error> {
    let path = Utf8WindowsPath::new(windows_path);
    if !path.is_absolute() {
        return Err(Error::RelativePathError);
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
                _ => {
                    return Err(Error::InvalidPrefixError);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_to_wsl() {
        assert_eq!(windows_to_wsl("C:\\Windows").unwrap(), "/mnt/c/Windows");
        assert_eq!(
            windows_to_wsl("C:\\foo\\..\\bar\\.\\baz.txt").unwrap(),
            "/mnt/c/bar/baz.txt"
        );
        assert_eq!(
            windows_to_wsl("C:\\Program Files (x86)\\Foo\\bar.txt").unwrap(),
            "/mnt/c/Program Files (x86)/Foo/bar.txt"
        );
    }

    #[test]
    fn test_windows_to_wsl_unc() {
        assert_eq!(
            windows_to_wsl("\\\\?\\C:\\Windows").unwrap(),
            "/mnt/c/Windows"
        );
        assert_eq!(
            windows_to_wsl("\\\\?\\C:\\foo\\..\\bar\\.\\baz.txt").unwrap(),
            "/mnt/c/bar/baz.txt"
        );
        assert_eq!(
            windows_to_wsl("\\\\?\\C:\\Program Files (x86)\\Foo\\bar.txt").unwrap(),
            "/mnt/c/Program Files (x86)/Foo/bar.txt"
        );
    }
}
