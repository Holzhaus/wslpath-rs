# wslpath-rs

This library aims to offer functionality similar of the `wslpath` conversion
tool [added in WSL build
17046](https://learn.microsoft.com/en-us/windows/wsl/release-notes#wsl-34), but
is implemented in pure Rust.

Existing crates such as [`wslpath`](https://crates.io/crates/wslpath) call
`wsl.exe wslpath` internally, which may lead to a lot of command invocations
when multiple paths need to be converted.

## License

This software is licensed under the terms of the [Mozilla Public License
2.0](https://www.mozilla.org/en-US/MPL/2.0/). Please also have a look at the
[license FAQ](https://www.mozilla.org/en-US/MPL/2.0/FAQ/).
