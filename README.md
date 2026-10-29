[github-license-mit]: https://github.com/FedericoBruzzone/tdlib-rs/blob/main/LICENSE-MIT
[github-license-apache]: https://github.com/FedericoBruzzone/tdlib-rs/blob/main/LICENSE-APACHE

# tdlib-rs

[![Latest version](https://img.shields.io/crates/v/tdlib_rs.svg)](https://crates.io/crates/tdlib_rs)
[![Documentation](https://docs.rs/tdlib-rs/badge.svg)](https://docs.rs/tdlib-rs/latest/tdlib_rs/)
[![Build](https://github.com/FedericoBruzzone/tdlib-rs/actions/workflows/build.yml/badge.svg)](https://github.com/FedericoBruzzone/tdlib-rs/actions/workflows/build.yml)
[![Test](https://github.com/FedericoBruzzone/tdlib-rs/actions/workflows/test.yml/badge.svg)](https://github.com/FedericoBruzzone/tdlib-rs/actions/workflows/test.yml)
[![Lint](https://github.com/FedericoBruzzone/tdlib-rs/actions/workflows/lint.yml/badge.svg)](https://github.com/FedericoBruzzone/tdlib-rs/actions/workflows/lint.yml)
[![downloads](https://img.shields.io/crates/d/tdlib_rs)](https://crates.io/crates/tdlib_rs)
![license](https://img.shields.io/crates/l/tdlib_rs)

A Rust wrapper around the Telegram Database library. It includes a generator to automatically generate the types and functions from the TDLib's [Type Language](https://core.telegram.org/mtproto/TL) file.

## Requirements

This crate supports **Linux only** and links against a **system-wide installation** of TDLib.
TDLib is never downloaded or bundled: building fails if `tdjson` cannot be found.

- `tdjson` (TDLib >= 1.8.68), installed system-wide, e.g. via your distribution's package manager
  or by building [TDLib](https://github.com/tdlib/td) and installing it with `cmake --install`.
- `pkg-config`, used at build time to locate `tdjson.pc`.

If TDLib is installed in a non-standard prefix, point `pkg-config` and the dynamic linker to it:

```sh
export PKG_CONFIG_PATH=/opt/tdlib/lib/pkgconfig:$PKG_CONFIG_PATH
export LD_LIBRARY_PATH=/opt/tdlib/lib:$LD_LIBRARY_PATH
```

## Information

It's mainly created for using it in the [tgt](https://github.com/FedericoBruzzone/tgt) client, but it should work also for any other Rust project.

Current supported TDLib version: [1.8.68](https://github.com/tdlib/td/commit/c15d3f5a5de6e3ba5839822c451152e5e18bb700).

## Cargo features

No feature is needed to build the library: just add it as a dependency.

```toml
[dependencies]
tdlib-rs = "..."
```

### docs

This feature skip the linking of the library and only generate the code of `generated.rs`.
Is used only for documentation and testing.

### bots-only-api

This feature enable the generation of the functions only used by Telegram bots.

## License

This repository are licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE][github-license-apache] or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT][github-license-mit] or <http://opensource.org/licenses/MIT>)

at your option.

Please review the license file provided in the repository for more information regarding the terms and conditions of the license.

## Contact

If you have any questions, suggestions, or feedback, do not hesitate to [contact me](https://federicobruzzone.github.io/).

Maintainers:

- [FedericoBruzzone](https://github.com/FedericoBruzzone)
- [Andreal2000](https://github.com/Andreal2000)

## Credits

- [grammers](https://github.com/Lonami/grammers): the `tdlib-tl-gen` and `tdlib-tl-parser` projects are forks of the `grammers-tl-gen` and `grammers-tl-parser` projects.
- [rust-tdlib](https://github.com/aCLr/rust-tdlib): for inspiration about some client code.
- [tdlib-rs](https://github.com/paper-plane-developers/tdlib-rs): for inspiration about the generator code.

