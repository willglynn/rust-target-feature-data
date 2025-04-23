//! Data about Rust target features.
//!
//! Different versions of Rust support different [target
//! features](https://rust-lang.github.io/rfcs/2045-target-feature.html). Different Rust targets
//! automatically enable different target features. Different versions of Rust automatically enable
//! different target features _for the same target_. Enabling the same target feature implies
// enabling different target features in different versions of Rust.
//!
//! This crate provides target feature data for all targets covering Rust versions:
//!
//! * `"1.85.0"` (1.85.1 is identical)
//! * `"1.86.0"`
//! * `"1.87.0"` (from 1.87.0-beta.5)
//!
//! Rust 1.88.0 provides target feature data for the selected target [via `rustdoc`'s JSON output
//! format](https://docs.rs/rustdoc-types/latest/rustdoc_types/struct.TargetFeature.html), making
//! this crate obsolete going forward.
//!
//! # Example
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use rust_target_feature_data::find;
//!
//! let features: Vec<_> = find("1.85.0", "i686-linux-android")?.collect();
//! let fxsr = features.iter().find(|f| f.name == "fxsr").unwrap();
//! assert_eq!(fxsr.globally_enabled, false);
//!
//! let features: Vec<_> = find("1.86.0", "i686-linux-android")?.collect();
//! let fxsr = features.iter().find(|f| f.name == "fxsr").unwrap();
//! assert_eq!(fxsr.globally_enabled, true);
//! # Ok(()) }
//! ```

use std::collections::BTreeSet;

#[rustfmt::skip]
mod generated;

/// Information about a target feature.
///
/// Rust target features are used to influence code generation, especially around selecting
/// instructions which are not universally supported by the target architecture.
///
/// Target features are commonly enabled by the [`#[target_feature]` attribute][1] to influence code
/// generation for a particular function, and less commonly enabled by compiler options like
/// `-Ctarget-feature` or `-Ctarget-cpu`. Targets themselves automatically enable certain target
/// features by default, for example because the target's ABI specification requires saving specific
/// registers which only exist in an architectural extension.
///
/// Target features can imply other target features: for example, x86-64 `avx2` implies `avx`, and
/// aarch64 `sve2` implies `sve`, since both of these architectural extensions depend on their
/// predecessors.
///
/// Target features can be probed at compile time by [`#[cfg(target_feature)]`][2] or `cfg!(…)`
/// conditional compilation to determine whether a target feature is enabled in a particular
/// context.
///
/// [1]: https://doc.rust-lang.org/stable/reference/attributes/codegen.html#the-target_feature-attribute
/// [2]: https://doc.rust-lang.org/reference/conditional-compilation.html#target_feature
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TargetFeature {
    /// The name of this target feature.
    pub name: &'static str,
    /// Other target features which are implied by this target feature, if any.
    pub implies_features: BTreeSet<&'static str>,
    /// If this target feature is unstable, the name of the associated language feature gate.
    pub unstable_feature_gate: Option<&'static str>,
    /// Whether this feature is globally enabled by default.
    ///
    /// Target features can be globally enabled implicitly as a result of the target's definition.
    /// For example, x86-64 hardware floating point ABIs require saving x87 and SSE2 registers,
    /// which in turn requires globally enabling the `x87` and `sse2` target features so that the
    /// generated machine code conforms to the target's ABI.
    ///
    /// Target features can also be globally enabled explicitly as a result of compiler flags like
    /// [`-Ctarget-feature`][1] or [`-Ctarget-cpu`][2].
    ///
    /// [1]: https://doc.rust-lang.org/beta/rustc/codegen-options/index.html#target-feature
    /// [2]: https://doc.rust-lang.org/beta/rustc/codegen-options/index.html#target-cpu
    pub globally_enabled: bool,
}

/// An error finding target feature data.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NotFoundError {
    /// The compiler version was not found
    CompilerNotFound(String),
    /// The compiler was found but the target was not
    TargetNotFound(String),
}

impl std::error::Error for NotFoundError {}

impl std::fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NotFoundError::CompilerNotFound(name) => {
                write!(f, "compiler version {:?} not found", name)
            }
            NotFoundError::TargetNotFound(name) => {
                write!(f, "target {:?} not found", name)
            }
        }
    }
}

/// Find the target features applicable to a Rust version and target.
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use rust_target_feature_data::NotFoundError;
///
/// // Different versions return different features
/// for (version, count) in [
/// //  ("1.83.0", 90),
/// //  ("1.84.0", 91),
///     ("1.85.0", 92),
///     ("1.86.0", 92),
///     ("1.87.0", 92),
/// ] {
///     assert_eq!(
///         rust_target_feature_data::find(version, "aarch64-apple-darwin")?.count(),
///         count,
///     );
/// }
///
/// // 1.84.0 data is not included
/// assert_eq!(
///     rust_target_feature_data::find("1.84.0", "x86_64-unknown-linux-gnu").err().unwrap(),
///     NotFoundError::CompilerNotFound("1.84.0".into())
/// );
///
/// // i686-unknown-redox became i586-unknown-redox
/// assert!(
///     rust_target_feature_data::find("1.85.0", "i686-unknown-redox").is_ok()
/// );
/// assert_eq!(
///     rust_target_feature_data::find("1.86.0", "i686-unknown-redox").err().unwrap(),
///     NotFoundError::TargetNotFound("i686-unknown-redox".into())
/// );
/// # Ok(()) }
/// ```
pub fn find(
    rust_version: &str,
    target: &str,
) -> Result<impl Iterator<Item = TargetFeature>, NotFoundError> {
    let mut targets = generated::all()
        .find(|(version, _)| *version == rust_version)
        .ok_or_else(|| NotFoundError::CompilerNotFound(rust_version.into()))?
        .1;

    targets
        .find(|(name, _)| *name == target)
        .map(|(_, features)| features)
        .ok_or_else(|| NotFoundError::TargetNotFound(target.into()))
}

#[cfg(test)]
mod tests;
