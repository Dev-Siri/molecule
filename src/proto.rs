use anyhow::bail;

#[derive(Debug)]
pub enum InputType {
    /// Gracefully shutdown the database.
    Stop,
}

impl TryFrom<&str> for InputType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "stop" => Self::Stop,
            _ => bail!("Invalid input type: {}", value),
        })
    }
}

impl TryFrom<String> for InputType {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(match value.as_str() {
            "stop" => Self::Stop,
            _ => bail!("Invalid input type: {}", value),
        })
    }
}
