// rustc-cfg emitted by the build script:
//
// "use_proc_macro"
//     Link to extern crate proc_macro. Available on any compiler and any target
//     except wasm32. Requires "proc-macro" Cargo cfg to be enabled (default is
//     enabled). On wasm32 we never link to proc_macro even if "proc-macro" cfg
//     is enabled.
//
// "wrap_proc_macro"
//     Wrap types from libproc_macro rather than polyfilling the whole API.
//     Enabled on rustc 1.29+ as long as procmacro2_semver_exempt is not set,
//     because we can't emulate the unstable API without emulating everything
//     else. Also enabled unconditionally on nightly, in which case the
//     procmacro2_semver_exempt surface area is implemented by using the
//     nightly-only proc_macro API.
//
// "hygiene"
//    Enable Span::mixed_site() and non-dummy behavior of Span::resolved_at
//    and Span::located_at. Enabled on Rust 1.45+.
//
// "proc_macro_span"
//     Enable non-dummy behavior of Span::start and Span::end methods which
//     requires an unstable compiler feature. Enabled when building with
//     nightly, unless `-Z allow-feature` in RUSTFLAGS disallows unstable
//     features.
//
// "super_unstable"
//     Implement the semver exempt API in terms of the nightly-only proc_macro
//     API. Enabled when using procmacro2_semver_exempt on a nightly compiler.
//
// "span_locations"
//     Provide methods Span::start and Span::end which give the line/column
//     location of a token. Enabled by procmacro2_semver_exempt or the
//     "span-locations" Cargo cfg. This is behind a cfg because tracking
//     location inside spans is a performance hit.
//
// "is_available"
//     Use proc_macro::is_available() to detect if the proc macro API is
//     available or needs to be polyfilled instead of trying to use the proc
//     macro API and catching a panic if it isn't available. Enabled on Rust
//     1.57+.
#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]


#[warn(non_upper_case_globals)]
#[cfg(not(target_env = "sgx"))]




extern crate sgx_tstd as std;
use std::prelude::v1::*;

use std::env;


use std::str;

fn main() {
    

    let version = match rustc_version() {
        Some(version) => version,
        None => return,
    };

    let docs_rs = env::var_os("DOCS_RS").is_some();
    let semver_exempt = cfg!(procmacro2_semver_exempt) || docs_rs;
    if semver_exempt {
        // https://github.com/dtolnay/proc-macro2/issues/147
       
    }

    if semver_exempt || cfg!(feature = "span-locations") {
       
    }


    let target = env::var("TARGET").unwrap();
    if !enable_use_proc_macro(&target) {
        return;
    }


}

fn enable_use_proc_macro(target: &str) -> bool {
    // wasm targets don't have the `proc_macro` crate, disable this feature.
    if target.contains("wasm32") {
        return false;
    }

    // Otherwise, only enable it if our feature is actually enabled.
    cfg!(feature = "proc-macro")
}

struct RustcVersion {
    minor: u32,
    nightly: bool,
}

fn rustc_version() -> Option<RustcVersion> {

    let nightly = true;
    let minor = 61;
    Some(RustcVersion { minor,nightly})
}

fn feature_allowed(feature: &str) -> bool {
    // Recognized formats:
    //
    //     -Z allow-features=feature1,feature2
    //
    //     -Zallow-features=feature1,feature2

    let flags_var;
    let flags_var_string;
    let flags = if let Some(encoded_rustflags) = env::var_os("CARGO_ENCODED_RUSTFLAGS") {
        flags_var = encoded_rustflags;
        flags_var_string = flags_var.to_string_lossy();
        flags_var_string.split('\x1f')
    } else {
        return true;
    };

    for mut flag in flags {
        if flag.starts_with("-Z") {
            flag = &flag["-Z".len()..];
        }
        if flag.starts_with("allow-features=") {
            flag = &flag["allow-features=".len()..];
            return flag.split(',').any(|allowed| allowed == feature);
        }
    }

    // No allow-features= flag, allowed by default.
    true
}
