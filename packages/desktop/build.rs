//! Lets `cargo check` / rust-analyzer work on a fresh checkout.
//!
//! `asset!("/assets/tailwind.css")` needs the file to exist at compile time,
//! but it is generated (Tailwind CLI, run by `dx` or `tailwind-build`) and
//! git-ignored. If it is missing we write an empty placeholder. `dx`, CI and
//! the Nix build overwrite it with the real stylesheet. Never replaces an
//! existing file.

use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets it"));
    let css = manifest_dir.join("assets").join("tailwind.css");
    if css.exists() {
        return;
    }

    let created = fs::create_dir_all(manifest_dir.join("assets")).and_then(|()| {
        fs::write(
            &css,
            "/* placeholder: real CSS comes from Tailwind (dx serve/bundle) */\n",
        )
    });
    match created {
        Ok(()) => println!(
            "cargo:warning=created an empty {}; run `dx serve`/`dx bundle` (or `tailwind-build`) for real styles",
            css.display()
        ),
        Err(e) => println!("cargo:warning=could not create {}: {e}", css.display()),
    }
}
