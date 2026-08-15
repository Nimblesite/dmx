//! Build inputs for the `dmx` crate [release.version].
//!
//! `DMX_VERSION` is what `dmx --version` reports, injected by the release from
//! the tag it is publishing. Cargo does not know a `option_env!` read happened,
//! so without this it would serve a cached binary carrying the previous
//! version — a release that silently claims to be the one before it.

fn main() {
    println!("cargo::rerun-if-env-changed=DMX_VERSION");
}
