# Rust target feature data

Different versions of Rust support different [target features](https://rust-lang.github.io/rfcs/2045-target-feature.html). Different Rust targets automatically enable different target features. Different versions of Rust automatically enable different target features _for the same target_.

`rustdoc` >= 1.88 outputs target feature information as JSON. Older versions do not, but I have backported this change:

* [1.83.0](https://github.com/willglynn/rust/commits/rustdoc_target_features_backport_v1.83.0/)
* [1.84.0](https://github.com/willglynn/rust/commits/rustdoc_target_features_backport_v1.84.0/)
* [1.84.1](https://github.com/willglynn/rust/commits/rustdoc_target_features_backport_v1.84.1/)
* [1.85.0](https://github.com/willglynn/rust/commits/rustdoc_target_features_backport_v1.85.0/)
* [1.85.1](https://github.com/willglynn/rust/commits/rustdoc_target_features_backport_v1.85.1/)
* [1.86.0](https://github.com/willglynn/rust/commits/rustdoc_target_features_backport_v1.86.0/)
* [1.86.0](https://github.com/willglynn/rust/commits/rustdoc_target_features_backport_v1.86.0/)
* [1.87.0-beta](https://github.com/willglynn/rust/commits/rustdoc_target_features_backport_v1.87.0-beta/)

This repository contains target feature data extracted from each of these compilers.
