use proc_macro2::TokenStream;
use quote::quote;
use rust_target_feature_data_dev as dev;
use std::collections::{BTreeMap, BTreeSet};
use std::iter;

trait VecExt<T> {
    fn push_once(&mut self, value: T) -> usize;
    fn find_once(&self, value: &T) -> Option<usize>;
}

impl<T: Eq> VecExt<T> for Vec<T> {
    fn push_once(&mut self, value: T) -> usize {
        if let Some((n, _)) = self.iter().enumerate().find(|(_, v)| **v == value) {
            n
        } else {
            self.push(value);
            self.len() - 1
        }
    }

    fn find_once(&self, value: &T) -> Option<usize> {
        for (i, v) in self.iter().enumerate() {
            if v == value {
                return Some(i);
            }
        }
        None
    }
}

fn main() {
    let compilers = dev::load().unwrap();

    // The main ideas here are to deduplicate identical data, and to present this data as something
    // that's fast and small to compile. Runtime performance doesn't matter here or in the generated
    // crate. The data we're trying to describe is static. Everything else is secondary.
    //
    // Concretely:
    //   1. Strings are duplicated. Factor them out into a `NAMES` array, and refer to strings by
    //      index. We don't have that many strings, so indices can be `u16`.
    //   2. Many features imply the same list of features. Factor those lists out into something we
    //      can `include_bytes!()`, and generate a `get_implies_features(n)` function.
    //   3. Many features are identical. Factor features out into something we can
    //      `include_bytes!()`, and generate a `get_feature(n)` function.
    //   4. Many targets refer to the same list of features. Factor feature lists out into something
    //      we can `include_bytes!()`, and generate a `get_feature_list(n)` function.
    //   5. Generate a `pub(crate) all()` function, iterating over compiler versions, iterating over
    //      targets, iterating over target features.

    // Accumulate unique features, their implications, and feature name strings
    let mut features = BTreeSet::new();
    let mut feature_implies_features = BTreeSet::new();
    let mut names = BTreeSet::new();
    for compiler in &compilers {
        for target in &compiler.targets {
            names.insert(target.triple.clone());
            for feature in &target.target_features {
                names.insert(feature.name.clone());
                if let Some(gate) = &feature.unstable_feature_gate {
                    names.insert(gate.clone());
                }
                features.insert(feature.clone());
                feature_implies_features.insert(feature.implies_features.clone());
            }
        }
    }

    // Convert into Vecs to get meaningful indices
    let features = Vec::from_iter(features);
    let feature_implies_features = Vec::from_iter(feature_implies_features);
    let names = Vec::from_iter(names);

    // Build a map of unique feature lists, and a map of (compiler, target) => feature list ID
    let mut feature_lists = Vec::new();
    let maps: BTreeMap<&str, BTreeMap<&str, usize>> = compilers
        .iter()
        .map(|compiler| {
            let ids = compiler
                .targets
                .iter()
                .map(|target| {
                    let feature_ids: Vec<usize> = target
                        .target_features
                        .iter()
                        .map(|f| features.find_once(f).unwrap())
                        .collect();

                    let id = feature_lists.push_once(feature_ids);
                    (target.triple.as_str(), id)
                })
                .collect();
            (compiler.version.as_str(), ids)
        })
        .collect();

    // Start preparing the output
    let mut output = quote! {
        use super::*;
    };

    // Output NAMES
    {
        let names: TokenStream = names.iter().map(|s| quote! { #s, }).collect();
        output.extend(quote! {
            static NAMES: &[&str] = &[#names];
        });
    }

    // Output `get_implies_features(n)` and supporting data
    {
        let implies_features_names: Vec<u16> = feature_implies_features
            .iter()
            .map(|list| {
                list.iter()
                    .map(|s| names.find_once(s).unwrap())
                    .map(|v| u16::try_from(v + 1).unwrap())
                    .chain(iter::once(0))
            })
            .flatten()
            .collect();

        let implies_features_names: TokenStream = implies_features_names
            .into_iter()
            .map(|x| quote! { #x,})
            .collect();
        output.extend(quote! {
            static IMPLIES_FEATURES_NAMES: &[u16] = &[#implies_features_names];

            fn get_implies_features(mut n: usize) -> impl Iterator<Item = &'static str> {
                let mut values = IMPLIES_FEATURES_NAMES.iter().skip_while(move |v| {
                    if n == 0 {
                        false
                    } else {
                        if **v == 0 {
                            n -= 1;
                        }
                        true
                    }
                });
                std::iter::from_fn(move || {
                    let id = *values.next().unwrap();
                    id.checked_sub(1).map(|idx| NAMES[idx as usize])
                })
                .fuse()
            }
        });
    }

    // Output `get_feature(n)` and supporting data
    {
        let features_parts: Vec<Vec<u16>> = features
            .iter()
            .map(|feature| {
                let unstable_feature_gate = u16::try_from(
                    feature
                        .unstable_feature_gate
                        .as_ref()
                        .map(|str| names.find_once(str).unwrap() + 1)
                        .unwrap_or(0),
                )
                .unwrap();
                let packed =
                    unstable_feature_gate | if feature.globally_enabled { 0x8000 } else { 0 };

                [
                    u16::try_from(names.find_once(&feature.name).unwrap()).unwrap(),
                    packed,
                    u16::try_from(
                        feature_implies_features
                            .find_once(&feature.implies_features)
                            .unwrap(),
                    )
                    .unwrap(),
                ]
                .into_iter()
                .collect::<Vec<u16>>()
            })
            .collect();

        let features_blob = features_parts
            .into_iter()
            .flatten()
            .map(|v| v.to_le_bytes())
            .flatten()
            .collect::<Vec<u8>>();
        std::fs::write("src/generated_features.blob", features_blob).unwrap();

        output.extend(quote! {
            static FEATURES_BLOB: &[u8] = include_bytes!("generated_features.blob");

            fn get_feature(n: usize) -> TargetFeature {
                let offset = n * 6;
                let blob = &FEATURES_BLOB[offset..];
                let mut list = blob.chunks(2).map(|bytes| u16::from_le_bytes(bytes.try_into().unwrap()));
                let name = NAMES[list.next().unwrap() as usize];
                let packed = list.next().unwrap();
                let unstable_feature_gate = (packed & 0x7fff).checked_sub(1).map(|idx| NAMES[idx as usize]);
                let globally_enabled = (packed & 0x8000) != 0;
                let implies_features = get_implies_features(list.next().unwrap() as usize).collect();
                TargetFeature {
                    name,
                    unstable_feature_gate,
                    globally_enabled,
                    implies_features,
                }
            }
        });
    }

    // Output `get_feature_list(n)` and supporting data
    {
        let feature_lists_parts: Vec<Vec<u16>> = feature_lists
            .iter()
            .map(|list| {
                list.iter()
                    .map(|v| u16::try_from(*v + 1).unwrap())
                    .chain(iter::once(0))
                    .collect::<Vec<u16>>()
            })
            .collect();

        let mut n = 0;
        let feature_lists_offsets: Vec<usize> = feature_lists_parts
            .iter()
            .map(|vec| {
                let my_n = n;
                n += vec.len() * 2;
                my_n
            })
            .collect();
        let feature_list_blob = feature_lists_parts
            .into_iter()
            .flatten()
            .map(|v| v.to_le_bytes())
            .flatten()
            .collect::<Vec<u8>>();
        std::fs::write("src/generated_feature_lists.blob", feature_list_blob).unwrap();

        let feature_lists_offsets: TokenStream = feature_lists_offsets
            .iter()
            .map(|x| quote! { #x,})
            .collect();
        output.extend(quote! {
            static FEATURE_LISTS_BLOB: &[u8] = include_bytes!("generated_feature_lists.blob");
            static FEATURE_LISTS_OFFSETS: &[usize] = &[#feature_lists_offsets];

            fn get_feature_list(n: usize) -> impl Iterator<Item=TargetFeature> {
                let offset = FEATURE_LISTS_OFFSETS[n];
                let blob = &FEATURE_LISTS_BLOB[offset..];
                let mut list = blob.chunks(2).map(|bytes| u16::from_le_bytes(bytes.try_into().unwrap()));
                std::iter::from_fn(move || {
                    list.next().unwrap().checked_sub(1).map(|id| get_feature(id as usize))
                }).fuse()
            }
        });
    }

    // Output `all()` and supporting data
    let target_maps_parts: Vec<_> = maps
        .iter()
        .map(|(version, targets)| {
            let u16s: Vec<u16> = targets
                .iter()
                .map(|(target, feature_list)| {
                    let target = String::from(*target);
                    let target = u16::try_from(names.find_once(&target).unwrap()).unwrap();
                    let feature_list = u16::try_from(*feature_list).unwrap();
                    [target, feature_list]
                })
                .flatten()
                .collect();
            (version, u16s)
        })
        .collect();

    let target_maps_offsets: Vec<(&str, usize)> = {
        let mut n = 0;
        target_maps_parts
            .iter()
            .map(|(ver, vec)| {
                let my_n = n;
                n += vec.len() * 2;
                (**ver, my_n)
            })
            .collect()
    };

    let target_maps_blob = target_maps_parts
        .into_iter()
        .map(|(_, vec)| vec)
        .flatten()
        .map(|v| v.to_le_bytes())
        .flatten()
        .collect::<Vec<u8>>();
    let target_maps_offsets: TokenStream = target_maps_offsets
        .iter()
        .chain(std::iter::once(&("", target_maps_blob.len())))
        .map(|(version, offset)| quote! { (#version, #offset),})
        .collect();
    std::fs::write("src/generated_target_maps.blob", target_maps_blob).unwrap();

    output.extend(quote! {
        static TARGET_MAPS_BLOB: &[u8] = include_bytes!("generated_target_maps.blob");
        static TARGET_MAPS_OFFSETS: &[(&str, usize)] = &[#target_maps_offsets];

        pub(crate) fn all() -> impl Iterator<Item=(&'static str, impl Iterator<Item=(&'static str, impl Iterator<Item=TargetFeature>)>)> {
            TARGET_MAPS_OFFSETS.windows(2).map(|window| {
                let &[(version, start), (_, end)] = window else {
                    unreachable!()
                };
                let slice = &TARGET_MAPS_BLOB[start..end];
                let targets = slice.chunks(4).map(|bytes| {
                    let target_name = u16::from_le_bytes([bytes[0], bytes[1]]);
                    let feature_list = u16::from_le_bytes([bytes[2], bytes[3]]);
                    (
                        NAMES[target_name as usize],
                        get_feature_list(feature_list.into()),
                    )
                });
                (version, targets)
            })
        }
    });

    // Pretty-print the file and write it to disk
    let file = syn::parse_file(&output.to_string()).unwrap();
    let generated = prettyplease::unparse(&file);
    let with_header = format!(
        "// WARNING: This file was generated automatically by rust-target-feature-data-gen.\n\n{}",
        generated
    );
    std::fs::write("src/generated.rs", with_header.into_bytes()).unwrap()
}
