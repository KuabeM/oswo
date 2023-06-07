use std::ops::{Deref, DerefMut};

use color_eyre::Result;
use swayipc::{Connection, Mode};

#[derive(Debug, Clone, PartialEq)]
pub struct Output {
    name: String,
    model: String,
    position: (i32, i32),
    resolution: (u32, u32),
    modes: Vec<Mode>,
}

impl Output {
    /// Creates a new [`Output`].
    fn new(
        name: String,
        model: String,
        position: (i32, i32),
        resolution: (u32, u32),
        modes: Vec<Mode>,
    ) -> Self {
        Self {
            name,
            model,
            position,
            resolution,
            modes,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn display(&self, verbose: bool, name_pad: usize) -> String {
        // we want at least one whitespace, hence + 1
        let pad = name_pad.saturating_sub(self.name.len()) + 1;
        let modes = self.modes.iter().fold(String::new(), |mut acc, m| {
            let refresh = m.refresh as f32 / 1000.0;
            acc = acc
                + ", "
                + &format!("{}x{} ({} Hz)", m.width, m.height, refresh);
            acc
        });

        let details = if verbose {
            ", modes: ".to_string() + &modes
        } else {
            "".to_string()
        };
        let resolution = format!("{}x{}", self.resolution.0, self.resolution.1);
        format!(
            "{}:{:0pad$}position: {:4}/{}, resolution: {:>9}, model: {}{}",
            self.name,
            " ",
            self.position.0,
            self.position.1,
            resolution,
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

    fn get_name_from_model<'a, 'b>(&'a self, model: &'a str) -> Option<&'a str>
    where
        'a: 'b,
    {
        self.0
            .iter()
            .find(|e| e.model.contains(model))
            .map(|o| o.name.as_ref())
    }

    pub fn set_models(&self, setup: &[String]) -> Result<()> {
        let names: Vec<String> = setup
            .iter()
            .map(|model| {
                self.get_name_from_model(model)
                    .map(|n| n.to_string())
                    .ok_or_else(|| color_eyre::eyre::eyre!("Failed to find name of '{}'", model))
            })
            .collect::<Result<Vec<_>>>()?;
        self.set(&names)
    }

    pub fn set(&self, setup: &[String]) -> Result<()> {
        let setup: Vec<String> = setup.iter().map(|s| s.to_lowercase()).collect();
        let desired: Outputs = setup
            .iter()
            .filter_map(|s| {
                if let Some(o) = self.iter().find(|o| o.name().to_lowercase() == *s) {
                    Some(o)
                } else {
                    println!("Display {} not connected", s);
                    None
                }
            })
            .collect();
        if desired.is_empty() {
            color_eyre::eyre::bail!("No display to be set is connected");
        }

        let mut cmd_con = swayipc::Connection::new()?;
        for o in self
            .iter()
            .filter(|o| !setup.contains(&o.name().to_string().to_lowercase()))
        {
            let payload = format!("output {} disable", o.name());
            cmd_con.run_command(payload)?;
        }

        let desired_modes: Vec<Option<&swayipc::Mode>> =
            desired.iter().map(|o| o.best_mode()).collect();

        let mut last_x = 0;
        for (output, mode) in desired.iter().zip(desired_modes) {
            let (width, height) = mode.map(|m| (m.width, m.height)).unwrap_or((0, 0));
            let payload = format!(
                "output {} enable position {} 0 resolution {}x{}",
                output.name(),
                last_x,
                width,
                height
            );
            last_x += width;
            println!("{payload}");
            cmd_con.run_command(payload)?;
        }

        Ok(())
    }
}

impl std::fmt::Display for Outputs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let verbose = f.alternate();
        let name_pad = self.longest_name();
        self.0.iter().fold(Ok(()), |r, output| {
            r.and_then(|_| writeln!(f, "{}", output.display(verbose, name_pad)))
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
            Vec::new(),
        );
        let display = output.display(false, 8);
        assert_eq!(&display[..10], "1234:     ");
    }
}
