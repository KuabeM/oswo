use color_eyre::{eyre::Context, Result};
use log::info;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ops::Deref,
    path::{Path, PathBuf},
};

use crate::Outputs;

#[derive(Debug, Deserialize)]
pub struct Cfgs(HashMap<String, Config>);

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DesiredOutput {
    pub name: String,
    pub scale: Option<f64>,
}

/// Config describes a named configuration: outputs plus optional priority
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub outputs: Vec<DesiredOutput>,
    /// higher number -> higher priority; optional for backwards compatibility
    pub priority: Option<i64>,
}

impl Deref for Cfgs {
    type Target = HashMap<String, Config>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// TODO: allow name + scale and name only
// #[derive(Debug, Deserialize)]
// enum OutputVariants {
//     Full(DesiredOutput),
//     Name(String),
// }

impl TryFrom<&toml_edit::Table> for Cfgs {
    type Error = color_eyre::Report;

    fn try_from(table: &toml_edit::Table) -> std::result::Result<Self, Self::Error> {
        let cfg: Result<HashMap<String, Config>> = table
            .into_iter()
            .map(|(name, inner)| {
                let section_str = inner
                    .as_table()
                    .map(|t| t.to_string())
                    .unwrap_or(inner.as_str().unwrap_or("").to_string());
                let cfg_entry: Config =
                    toml_edit::de::from_str(&section_str).wrap_err_with(|| {
                        format!(
                            "Missing outputs in configuration {}: {}",
                            &name,
                            &inner.to_string(),
                        )
                    })?;
                let name = name.to_string();
                Ok((name, cfg_entry))
            })
            .collect();
        Ok(Cfgs(cfg?))
    }
}

impl Cfgs {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let cfg_str = std::fs::read_to_string(&path)
            .wrap_err_with(|| format!("Failed to read {}", path.as_ref().display()))?;
        let cfgs_doc: toml_edit::Document = cfg_str
            .parse()
            .wrap_err("Failed to parse configurtion file")?;
        let cfgs = cfgs_doc.as_table();
        Self::try_from(cfgs)
    }

    /// Return the Config for a named configuration (if present)
    pub fn find(&self, key: &str) -> Option<&Config> {
        self.0.get(key)
    }

    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or("/etc/xdg/".into())
            .join("oswo.toml")
    }

    pub fn add(&mut self, name: &str, outputs: &Outputs) -> Result<()> {
        let active_outputs: Vec<_> = outputs
            .iter()
            .filter(|o| o.enabled())
            .map(|o| DesiredOutput {
                name: o.name().to_string(),
                scale: Some(o.scale()),
            })
            .collect();

        match self.0.insert(
            name.to_string(),
            Config {
                outputs: active_outputs,
                priority: None,
            },
        ) {
            Some(_) => info!("Updated config {name}"),
            None => info!("Added new config {name}"),
        }
        Ok(())
    }

    /// Save the current configurations to the given path, overwriting the file.
    /// This is a simple overwrite: it serializes the internal HashMap to TOML and writes
    /// it atomically by writing to a temporary file then renaming.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();

        // Read existing document if present, otherwise start a new one.
        let mut doc = if path.exists() {
            let s = std::fs::read_to_string(path)
                .wrap_err_with(|| format!("Failed to read {}", path.display()))?;
            s.parse::<toml_edit::Document>()
                .wrap_err("Failed to parse existing TOML file")?
        } else {
            toml_edit::Document::new()
        };

        // For each config, build a table with outputs (array of inline tables) and optional priority,
        // then insert/replace into the document. This preserves other top-level content and comments.
        for (name, cfg) in &self.0 {
            let mut section = toml_edit::Table::new();

            // Build outputs array with inline tables
            let mut outputs_array = toml_edit::Array::new();
            for output in &cfg.outputs {
                let mut output_table = toml_edit::InlineTable::new();
                output_table.insert("name", output.name.clone().into());
                if let Some(scale) = output.scale {
                    output_table.insert("scale", scale.into());
                }
                outputs_array.push(output_table);
            }
            section["outputs"] = toml_edit::Item::Value(toml_edit::Value::Array(outputs_array));

            if let Some(p) = cfg.priority {
                section["priority"] = toml_edit::value(p);
            } else {
                // Ensure no stray priority remains if previously present: leave absent.
            }

            doc[name.as_str()] = toml_edit::Item::Table(section);
        }

        // Write atomically: write to tmp file then rename
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, doc.to_string())
            .wrap_err_with(|| format!("Failed to write temp file {}", tmp.display()))?;
        std::fs::rename(&tmp, path).wrap_err_with(|| {
            format!("Failed to rename {} -> {}", tmp.display(), path.display())
        })?;

        Ok(())
    }
}

impl std::fmt::Display for Cfgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().try_fold((), |_, (name, cfg)| {
            let setup_str = cfg
                .outputs
                .iter()
                .map(|o| format!("{}", o))
                .collect::<Vec<_>>()
                .join("\n  ");
            let priority_str = cfg
                .priority
                .map(|p| format!(" (priority: {})", p))
                .unwrap_or_default();
            write!(f, "{}{}:\n  {}\n", name, priority_str, setup_str)
        })
    }
}

impl std::fmt::Display for DesiredOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (scale: {})", self.name, self.scale.unwrap_or(1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_priority() {
        let s = r#"
        [a]
        outputs = [{ name = "Foo", scale = 1.0 }]
        priority = 5
        "#;
        let doc: toml_edit::Document = s.parse().unwrap();
        let cfgs = Cfgs::try_from(doc.as_table()).unwrap();
        let cfg = cfgs.find("a").expect("config 'a' present");
        assert_eq!(cfg.priority.unwrap(), 5);
        assert_eq!(cfg.outputs.len(), 1);
        assert_eq!(cfg.outputs[0].name, "Foo");
    }
}
