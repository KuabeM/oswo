use color_eyre::Result;
use swayipc::Connection;

#[derive(Debug)]
struct Output {
    name: String,
    model: String,
    position: (i32, i32),
    resolution: (u32, u32),
}

impl Output {
    fn new(name: String, model: String, position: (i32, i32), resolution: (u32, u32)) -> Self {
        Self {
            name,
            model,
            position,
            resolution,
        }
    }
}

#[derive(Debug)]
pub struct Outputs(Vec<Output>);

impl Outputs {
    pub fn list() -> Result<Self> {
        let raw_outputs = Connection::new()?.get_outputs()?;

        let outputs = raw_outputs
            .iter()
            .filter(|o| o.active)
            .map(|o| {
                let resolution = o
                    .current_mode
                    .and_then(|m| Some((m.width as u32, m.height as u32)))
                    .unwrap_or((0, 0));
                let output = Output::new(
                    o.name.clone(),
                    o.model.clone(),
                    (o.rect.x, o.rect.y),
                    resolution,
                );
                output
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
}

impl Output {
    fn display(&self, verbose: bool, name_pad: usize) -> String {
        // we want at least one whitespace, hence + 1
        let pad = name_pad - self.name.len() + 1;
        format!(
            "{}:{:0pad$}position: {:4}/{:4}, resolution: {}x{}",
            self.name, " ", self.position.0, self.position.1, self.resolution.0, self.resolution.1
        )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn padding() {
        let output = Output::new("1234".to_owned(), (0, 0), (0, 0));
        let display = output.display(8);
        assert_eq!(&display[..10], "1234:     ");
    }
}
