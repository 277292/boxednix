// SPDX-License-Identifier: GPL-3.0-only

use resolve_path::PathResolveExt;
use serde::{Deserialize, Deserializer, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_pathbuf_resolve")]
    pub identity: PathBuf,

    #[serde(
        default = "empty_pathbuf_vec",
        deserialize_with = "deserialize_pathbuf_vec_resolve",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub recipients: Vec<PathBuf>,

    #[serde(default = "empty_pathbuf_vec", skip_serializing_if = "Vec::is_empty")]
    pub recipients_files: Vec<PathBuf>,

    pub generate: Generate,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Generate {
    pub dir: PathBuf,
    pub gitignore: Gitignore,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Gitignore {
    Always,
    None,
}

fn empty_pathbuf_vec() -> Vec<PathBuf> {
    Vec::new()
}

fn deserialize_pathbuf_resolve<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    resolve_path::<D::Error>(s)
}

fn deserialize_pathbuf_vec_resolve<'de, D>(deserializer: D) -> Result<Vec<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    let list = Vec::<String>::deserialize(deserializer)?;
    list.into_iter().map(resolve_path).collect()
}

fn resolve_path<E>(s: String) -> Result<PathBuf, E>
where
    E: serde::de::Error,
{
    s.try_resolve()
        .map(|cow| cow.into_owned())
        .map_err(E::custom)
}
