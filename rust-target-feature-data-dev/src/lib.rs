#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Ord, Eq, PartialOrd, PartialEq, serde::Deserialize)]
pub struct Compiler {
    pub version: String,
    pub targets: BTreeSet<Target>,
}

#[derive(Debug, Clone, Ord, Eq, PartialOrd, PartialEq, serde::Deserialize)]
pub struct Target {
    pub triple: String,
    pub target_features: BTreeSet<TargetFeature>,
}

#[derive(Debug, Clone, Ord, Eq, PartialOrd, PartialEq, serde::Deserialize)]
pub struct TargetFeature {
    pub name: String,
    pub unstable_feature_gate: Option<String>,
    pub implies_features: BTreeSet<String>,
    pub globally_enabled: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum LoadError {
    #[error("reading directory: {0}")]
    ReadDirectory(std::io::Error),
    #[error("reading file: {0}")]
    ReadFile(std::io::Error),
    #[error("deserializing target {0}: {1}")]
    Deserialize(PathBuf, serde_json::Error),
}

pub fn load() -> Result<BTreeSet<Compiler>, LoadError> {
    [
        ("1.85.0", "1.85.0"),
        ("1.86.0", "1.86.0"),
        ("1.87.0", "1.87.0-beta.5"),
    ]
    .into_iter()
    .map(|(name, path)| {
        let path = PathBuf::from("data").join(path);
        load_compiler(name, path)
    })
    .collect()
}

fn load_compiler(version: &str, path: PathBuf) -> Result<Compiler, LoadError> {
    let entries = fs::read_dir(path)
        .map_err(LoadError::ReadDirectory)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(LoadError::ReadDirectory)?;

    let mut targets: BTreeSet<Target> = Default::default();

    for entry in entries {
        let file_name = entry.file_name();
        let Some(name) = file_name.as_os_str().to_str() else {
            continue;
        };
        if !name.ends_with(".json") {
            continue;
        }

        targets.insert(load_target(entry.path())?);
    }

    Ok(Compiler {
        version: version.into(),
        targets,
    })
}

fn load_target(path: PathBuf) -> Result<Target, LoadError> {
    let bytes: Vec<u8> = std::fs::read(&path).map_err(LoadError::ReadFile)?;
    serde_json::from_slice(&bytes).map_err(|e| LoadError::Deserialize(path, e))
}
