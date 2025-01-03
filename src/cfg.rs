use color_eyre::{eyre::Context, Result};
use serde::Deserialize;
use std::{
    collections::HashMap,
    ops::Deref,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
pub struct Cfgs(HashMap<String, Vec<DesiredOutput>>);

#[derive(Debug, Deserialize)]
struct Outputs {
    pub outputs: Vec<DesiredOutput>,
}

impl Deref for Cfgs {
    type Target = HashMap<String, Vec<DesiredOutput>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DesiredOutput {
    pub name: String,
    pub scale: Option<f64>,
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
        let cfg: Result<HashMap<String, Vec<DesiredOutput>>> = table
            .into_iter()
            .map(|(name, inner)| {
                let output_str = inner
                    .as_table()
                    .map(|t| t.to_string())
                    .unwrap_or(inner.as_str().unwrap_or("").to_string());
                let out: Outputs = toml_edit::de::from_str(&output_str).wrap_err_with(|| {
                    format!(
                        "Missing outputs in configuration {}: {}",
                        &name,
                        &inner.to_string(),
                    )
                })?;
                let name = name.to_string();
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
        let cfgs_doc: toml_edit::Document = cfg_str
            .parse()
            .wrap_err("Failed to parse configurtion file")?;
        let cfgs = cfgs_doc.as_table();
        Self::try_from(cfgs)
    }

    pub fn find(&self, key: &str) -> Option<&Vec<DesiredOutput>> {
        self.0.get(key)
    }

    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or("/etc/xdg/".into())
            .join("oswo.toml")
    }
}

impl std::fmt::Display for Cfgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().try_fold((), |_, (name, setup)| {
            let setup_str = setup
                .iter()
                .map(|o| format!("{}", o))
                .collect::<Vec<_>>()
                .join("\n  ");
            write!(f, "{}:\n  {}\n", name, setup_str)
        })
    }
}

impl std::fmt::Display for DesiredOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (scale: {})", self.name, self.scale.unwrap_or(1.0))
    }
}
