# tdlib-rs

A Rust wrapper around the Telegram Database library. It includes a generator to automatically generate the types and functions from the TDLib's [Type Language](https://core.telegram.org/mtproto/TL) file.

## Requirements

- `tdjson` (TDLib >= 1.8.68), installed system-wide, e.g. via your distribution's package manager or by building [TDLib](https://github.com/tdlib/td) and installing it with `cmake --install`.
- `pkg-config`, used at build time to locate `tdjson.pc`.
