# `rust-target-feature-data`

Different versions of Rust support different [target
features](https://rust-lang.github.io/rfcs/2045-target-feature.html). Different Rust targets
automatically enable different target features. Different versions of Rust automatically enable
different target features _for the same target_. Enabling the same target feature implies
enabling different target features in different versions of Rust.

This crate provides target feature data for all targets covering Rust versions:

* `"1.85.0"` (1.85.1 is identical)
* `"1.86.0"`
* `"1.87.0"` (from 1.87.0-beta.5)

Rust 1.88.0 provides target feature data for the selected target [via `rustdoc`'s JSON output
format](https://docs.rs/rustdoc-types/latest/rustdoc_types/struct.TargetFeature.html), making
this crate obsolete going forward.

# Example

```rust
use rust_target_feature_data::find;

let features: Vec<_> = find("1.85.0", "i686-linux-android")?.collect();
let fxsr = features.iter().find(|f| f.name == "fxsr").unwrap();
assert_eq!(fxsr.globally_enabled, false);

let features: Vec<_> = find("1.86.0", "i686-linux-android")?.collect();
let fxsr = features.iter().find(|f| f.name == "fxsr").unwrap();
assert_eq!(fxsr.globally_enabled, true);
```

# Development

This crate was optimized for compile time and compiled code size. It uses bespoke, artisan data
structures (gross, unreadable binary blobs) handcrafted (generated) by the finest tradespeople
(ugliest automation). Update `data/`, `rust-target-feature-data-dev`, and/or
`rust-target-feature-data-gen` if you must. Then:

```console
% cargo run -p rust-target-feature-data-gen
…
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.87s
     Running `target/debug/rust-target-feature-data-gen`
% cargo test
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.87s
     Running unittests src/lib.rs (target/debug/deps/rust_target_feature_data-751168831ae8de8d)

running 4 tests
test tests::compiler_not_found ... ok
test tests::target_not_found ... ok
test tests::smoke ... ok
test tests::compare_all ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s

   Doc-tests rust_target_feature_data

running 2 tests
test src/lib.rs - (line 21) ... ok
test src/lib.rs - find (line 111) ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
% git commit .
```

The generator tool runs offline, i.e. in development, and its output is committed to this
repository. Both the `-dev` nor `-gen` crates are purely internal to this workspace.

This crate's repository contains data for earlier Rust versions, but this data is not currently
required by any user and so is not included in the crate. If you have need for earlier data via
this crate, please open an issue.
