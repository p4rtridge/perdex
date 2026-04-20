use std::{error::Error, fmt};

#[derive(Debug)]
pub struct AppError;

impl Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "An error occurred during application execution")
    }
}
