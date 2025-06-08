use std::str::FromStr;
use crate::CliError;

#[derive(Default)]
pub enum CliFlag {
    #[default]
    Absent,
    Present,
}


impl FromStr for CliFlag {
    type Err = CliError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "present" | "pres" | "yes" | "true" | "1" | "y" => Ok(CliFlag::Present),
            "absent" | "abs" | "no" | "false" | "0" | "n" => Ok(CliFlag::Absent),
            _ => Err(CliError::UnsupportedFormat {
                format: s.to_string(),
                available: vec!["present", "absent"],
            }),
        }
    }
}

impl std::fmt::Display for CliFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliFlag::Present => write!(f, "present"),
            CliFlag::Absent => write!(f, "absent"),
        }
    }
}
