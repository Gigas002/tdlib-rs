// Copyright 2020 - developers of the `grammers` project.
// Copyright 2021 - developers of the `tdlib-rs` project.
// Copyright 2024 - developers of the `tgt` and `tdlib-rs` projects.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::env;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use tdlib_rs_gen::generate_rust_code;
use tdlib_rs_parser::parse_tl_file;
use tdlib_rs_parser::tl::Definition;

/// Load the type language definitions from a certain file.
/// Parse errors will be printed to `stderr`, and only the
/// valid results will be returned.
fn load_tl(file: &str) -> std::io::Result<Vec<Definition>> {
    let mut file = File::open(file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(parse_tl_file(contents)
        .filter_map(|d| match d {
            Ok(d) => Some(d),
            Err(e) => {
                eprintln!("TL: parse error: {e:?}");
                None
            }
        })
        .collect())
}

fn main() -> std::io::Result<()> {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os != "linux" {
        panic!("tdlib-rs only supports Linux targets, found: {target_os}");
    }

    println!("cargo:rerun-if-changed=build.rs");

    // Prevent linking libraries to avoid documentation failure
    #[cfg(not(feature = "docs"))]
    {
        // Probe the system-wide tdjson installation through pkg-config
        if let Err(e) = system_deps::Config::new().probe() {
            panic!("{e}\n\ntdlib-rs requires a system-wide installation of TDLib (tdjson)");
        }
    }

    let out_dir = env::var("OUT_DIR").unwrap();

    let definitions = load_tl("tl/api.tl")?;

    let mut file = BufWriter::new(File::create(Path::new(&out_dir).join("generated.rs"))?);

    generate_rust_code(&mut file, &definitions, cfg!(feature = "bots-only-api"))?;

    file.flush()?;

    Ok(())
}
