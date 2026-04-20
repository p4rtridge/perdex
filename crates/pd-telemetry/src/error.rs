use std::{error::Error, fmt};

#[derive(Debug)]
pub struct TelemetryError;

impl Error for TelemetryError {}

impl fmt::Display for TelemetryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "An error occurred during telemetry processing")
    }
}
