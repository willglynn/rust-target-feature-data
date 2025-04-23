`rustdoc` >= 1.88 outputs target feature information as JSON. Older versions do not, except via backports:

* [1.81.0](https://github.com/willglynn/rust/commits/rustdoc_target_features_backport_v1.81.0/) (no `implied_features`)
* [1.82.0](https://github.com/willglynn/rust/commits/rustdoc_target_features_backport_v1.82.0/)
* [1.83.0](https://github.com/willglynn/rust/commits/rustdoc_target_features_backport_v1.83.0/)
* [1.84.0](https://github.com/willglynn/rust/commits/rustdoc_target_features_backport_v1.84.0/)
* [1.84.1](https://github.com/willglynn/rust/commits/rustdoc_target_features_backport_v1.84.1/)
* [1.85.0](https://github.com/willglynn/rust/commits/rustdoc_target_features_backport_v1.85.0/)
* [1.85.1](https://github.com/willglynn/rust/commits/rustdoc_target_features_backport_v1.85.1/)
* [1.86.0](https://github.com/willglynn/rust/commits/rustdoc_target_features_backport_v1.86.0/)
* [1.86.0](https://github.com/willglynn/rust/commits/rustdoc_target_features_backport_v1.86.0/)
* [1.87.0-beta](https://github.com/willglynn/rust/commits/rustdoc_target_features_backport_v1.87.0-beta/)

This directory contains `{"target":}` data extracted from these patched compilers.
