use anyhow::{Context, Result, bail};
use rust_target_feature_data_dev as dev;
use std::collections::BTreeSet;

use super::*;

#[test]
fn smoke() {
    let features: Vec<_> = find("1.86.0", "x86_64-unknown-linux-gnu")
        .unwrap()
        .collect();

    let avx2 = features.iter().find(|f| f.name == "avx2").unwrap();
    assert_eq!(avx2.globally_enabled, false);
    assert!(
        avx2.implies_features
            .iter()
            .find(|f| **f == "avx")
            .is_some()
    );

    let sse2 = features.iter().find(|f| f.name == "sse2").unwrap();
    assert_eq!(sse2.globally_enabled, true);
    assert!(
        sse2.implies_features
            .iter()
            .find(|f| **f == "sse")
            .is_some()
    );
    assert!(
        sse2.implies_features
            .iter()
            .find(|f| **f == "avx2")
            .is_none()
    );
}

#[test]
fn compiler_not_found() {
    assert_eq!(
        find("1.0.0", "x86_64-unknown-linux-gnu").err().unwrap(),
        NotFoundError::CompilerNotFound("1.0.0".into())
    );
}

#[test]
fn target_not_found() {
    assert_eq!(
        find("1.86.0", "mos-c64-none").err().unwrap(),
        NotFoundError::TargetNotFound("mos-c64-none".into())
    );
}

#[test]
fn compare_all() {
    for compiler in rust_target_feature_data_dev::load().unwrap() {
        for target in compiler.targets {
            compare(&compiler.version, &target.triple, &target.target_features)
                .with_context(|| format!("comparing {} {}", compiler.version, &target.triple))
                .unwrap();
        }
    }
}

fn compare(compiler: &str, target: &str, expected: &BTreeSet<dev::TargetFeature>) -> Result<()> {
    let actual: Vec<_> = crate::find(compiler, target).context("finding")?.collect();
    if actual.len() != expected.len() {
        bail!("actual and expected are not the same length");
    }
    for (actual, expected) in actual.into_iter().zip(expected.into_iter()) {
        if actual.name != expected.name {
            bail!("actual name: {}, expected: {}", actual.name, expected.name);
        }
        if actual.globally_enabled != expected.globally_enabled {
            bail!(
                "actual globally_enabled: {}, expected: {}",
                actual.globally_enabled,
                expected.globally_enabled
            );
        }
        if actual.unstable_feature_gate
            != expected.unstable_feature_gate.as_ref().map(String::as_str)
        {
            bail!(
                "actual unstable_feature_gate: {:?}, expected: {:?}",
                actual.unstable_feature_gate,
                expected.unstable_feature_gate
            );
        }

        let actual_implies_features: Vec<&str> = actual.implies_features.iter().copied().collect();
        let expected_implies_features: Vec<&str> = expected
            .implies_features
            .iter()
            .map(String::as_str)
            .collect();

        if actual_implies_features != expected_implies_features {
            bail!(
                "actual implies_features: {:?}, expected: {:?}",
                actual_implies_features,
                expected_implies_features
            );
        }
    }

    Ok(())
}
