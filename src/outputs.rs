use std::ops::{Deref, DerefMut};

use color_eyre::Result;
use swayipc::{Connection, Mode};

use crate::cfg::DesiredOutput;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Output {
    name: String,
    model: String,
    position: (i32, i32),
    resolution: (u32, u32),
    scale: f64,
    enabled: bool,
    modes: Vec<Mode>,
}

impl Output {
    /// Creates a new [`Output`].
    fn new(
        name: String,
        model: String,
        position: (i32, i32),
        resolution: (u32, u32),
        scale: f64,
        enabled: bool,
        modes: Vec<Mode>,
    ) -> Self {
        Self {
            name,
            model,
            position,
            resolution,
            scale,
            enabled,
            modes,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn enabled(self) -> Self {
        Self {
            enabled: true,
            ..self
        }
    }

    pub fn with_scale(self, scale: f64) -> Self {
        Self { scale, ..self }
    }

    pub fn disabled(self) -> Self {
        Self {
            enabled: false,
            ..self
        }
    }

    fn display(&self, verbose: bool, name_pad: usize) -> String {
        // we want at least one whitespace, hence + 1
        let pad = name_pad.saturating_sub(self.name.len()) + 1;
        let modes = self.modes.iter().fold(String::new(), |mut acc, m| {
            let refresh = m.refresh as f32 / 1000.0;
            acc = acc + ", " + &format!("{}x{} ({} Hz)", m.width, m.height, refresh);
            acc
        });

        let details = if verbose {
            ", modes: ".to_string() + &modes
        } else {
            "".to_string()
        };
        let resolution = format!("{}x{}", self.resolution.0, self.resolution.1);
        format!(
            "{}:{:0pad$}position: {:4}/{}, resolution: {:>9}, scale: {:1.1}, model: {}{}",
            self.name,
            " ",
            self.position.0,
            self.position.1,
            resolution,
            self.scale,
            self.model.as_str(),
            details
        )
    }

    pub fn best_mode(&'_ self) -> Option<&'_ Mode> {
        self.modes
            .iter()
            .max_by_key(|mode| mode.width * mode.height)
    }
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt = self.display(false, 0);
        write!(f, "{}", fmt)
    }
}

#[derive(Debug, PartialEq)]
pub struct Outputs(Vec<Output>);

impl Outputs {
    pub fn list() -> Result<Self> {
        let raw_outputs = Connection::new()?.get_outputs()?;

        let outputs = raw_outputs
            .iter()
            .map(|o| {
                let resolution = o
                    .current_mode
                    .map(|m| (m.width as u32, m.height as u32))
                    .unwrap_or((0, 0));
                let model = o.make.clone() + " " + &o.model;
                Output::new(
                    o.name.clone(),
                    model,
                    (o.rect.x, o.rect.y),
                    resolution,
                    o.scale.unwrap_or(1.0),
                    o.active,
                    o.modes.clone(),
                )
            })
            .collect();

        let outputs = Self(outputs);
        Ok(outputs)
    }

    fn longest_name(&self) -> usize {
        self.0
            .iter()
            .fold(0, |len, output| len.max(output.name.len()))
    }

    pub fn set_models(&self, setup: &[DesiredOutput]) -> Result<()> {
        let disable: Vec<Output> = self
            .0
            .iter()
            .filter_map(|o| {
                if !setup.iter().any(|d| d.name == o.model) {
                    Some(o.clone().disabled())
                } else {
                    None
                }
            })
            .collect();

        let new_setup: Result<Vec<Output>> = setup
            .iter()
            .map(|desired| {
                self.0
                    .iter()
                    .find(|o| o.model == desired.name)
                    .ok_or(color_eyre::eyre::eyre!(
                        "Display '{}' is not connected",
                        desired.name
                    ))
                    .map(|o| o.clone().enabled().with_scale(desired.scale.unwrap_or(1.0)))
            })
            .collect();
        let new_setup = new_setup?;
        self.set(new_setup.iter())?;
        self.set(disable.iter())
    }

    pub fn set_by_name(&self, setup: &[String]) -> Result<()> {
        let outputs: Vec<_> = self
            .0
            .iter()
            .map(|o| {
                if setup.iter().any(|desired| **desired == o.name) {
                    o.clone().enabled()
                } else {
                    o.clone().disabled()
                }
            })
            .collect();
        self.set(outputs.iter())
    }

    fn set<'a>(&self, new_setup: impl Iterator<Item = &'a Output>) -> Result<()> {
        let mut cmd_con = swayipc::Connection::new()?;
        let mut last_x = 0;
        for o in new_setup {
            let payload = if o.enabled {
                let desired_mode = o.best_mode();
                let (width, height) = desired_mode.map(|m| (m.width, m.height)).unwrap_or((0, 0));
                let payload = format!(
                    "output {} enable position {} 0 resolution {}x{} scale {}",
                    o.name(),
                    last_x,
                    width,
                    height,
                    o.scale
                );
                last_x += width;
                payload
            } else {
                format!("output {} disable", o.name())
            };
            println!("{}", payload);
            cmd_con.run_command(payload)?;
        }

        Ok(())
    }
}

impl std::fmt::Display for Outputs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let verbose = f.alternate();
        let name_pad = self.longest_name();
        self.0.iter().try_fold((), |_, output| {
            writeln!(f, "{}", output.display(verbose, name_pad))
        })
    }
}

impl Deref for Outputs {
    type Target = Vec<Output>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Outputs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> FromIterator<&'a Output> for Outputs {
    fn from_iter<T: IntoIterator<Item = &'a Output>>(iter: T) -> Self {
        let mut vec: Vec<Output> = Vec::new();
        for n in iter {
            vec.push(n.clone());
        }

        Self(vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn padding() {
        let output = Output::new(
            "1234".to_owned(),
            "model".to_owned(),
            (0, 0),
            (0, 0),
            1.0,
            true,
            Vec::new(),
        );
        let display = output.display(false, 8);
        assert_eq!(&display[..10], "1234:     ");
    }
}
