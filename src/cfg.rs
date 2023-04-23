use color_eyre::{eyre::Context, Result};
use serde::Deserialize;
use std::{collections::HashMap, path::Path};

#[derive(Debug, Deserialize)]
pub struct Cfgs(HashMap<String, Vec<String>>);

#[derive(Debug, Deserialize)]
struct Outputs {
    pub outputs: Vec<String>,
}

impl TryFrom<toml::Table> for Cfgs {
    type Error = color_eyre::Report;

    fn try_from(table: toml::Table) -> std::result::Result<Self, Self::Error> {
        let cfg: Result<HashMap<String, Vec<String>>> = table
            .into_iter()
            .map(|(name, inner)| {
                let output_str = inner
                    .as_table()
                    .map(|t| t.to_string())
                    .unwrap_or(inner.as_str().unwrap_or("").to_string());
                let out: Outputs = toml::from_str(&output_str).wrap_err_with(|| {
                    format!(
                        "Missing outputs in configuration {}: {}",
                        &name,
                        &inner.to_string(),
                    )
                })?;
                Ok((name, out.outputs))
            })
            .collect();
        Ok(Cfgs(cfg?))
    }
}

impl Cfgs {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let cfg_str = std::fs::read_to_string(&path)
            .wrap_err_with(|| format!("Failed to read {}", path.as_ref().display()))?;
        let cfgs: toml::Table =
            toml::from_str(&cfg_str).wrap_err("Failed to parse configurtion file")?;
        Self::try_from(cfgs)
    }

    pub fn find(&self, key: &str) -> Option<&Vec<String>> {
        self.0.get(key)
    }
}
