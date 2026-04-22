use std::fmt::Write;

use error_stack::{Report, ResultExt};
use thiserror::Error;

use crate::cavage::SignatureHeader;

mod lexer;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Failed to build signature header")]
    BuildFailed,

    #[error("Invalid header format")]
    InvalidHeaderFormat,

    #[error("Failed to parse radix-10 value")]
    Radix10Parse,
}

/// Parse a signature header string into a `SignatureHeader` struct
pub fn parse(
    input: &str,
) -> Result<SignatureHeader<'_, impl Iterator<Item = &'_ str> + Clone>, Report<ParseError>> {
    let mut tokenizer = lexer::tokenize(input);

    let mut builder = SignatureHeader::builder();
    while let Some((key, value)) = tokenizer
        .next()
        .transpose()
        .change_context(ParseError::InvalidHeaderFormat)?
    {
        match key {
            "keyId" => builder.key_id(value),
            "signature" => builder.signature(value),
            "headers" => builder.headers(value.split_whitespace()),
            "created" => builder.created(
                value
                    .parse::<u64>()
                    .change_context(ParseError::Radix10Parse)?,
            ),
            "expires" => builder.expires(
                value
                    .parse::<u64>()
                    .change_context(ParseError::Radix10Parse)?,
            ),
            // Ignore unknown fields
            _ => {}
        }
    }

    builder.build().change_context(ParseError::BuildFailed)
}

/// Serialize a `SignatureHeader` struct into its string representation
pub fn serialize<'a, I>(header: SignatureHeader<'_, I>) -> String
where
    I: Iterator<Item = &'a str>,
{
    let mut buffer = String::new();

    let _ = write!(buffer, "keyId=\"{}\"", header.key_id);

    buffer.push_str(",headers=\"");
    for (i, item) in header.headers.enumerate() {
        if i > 0 {
            buffer.push(' ');
        }
        buffer.push_str(item);
    }
    buffer.push('"');

    let _ = write!(buffer, ",signature=\"{}\"", header.signature);

    if let Some(created) = header.created {
        let _ = write!(buffer, ",created={created}");
    }

    if let Some(expires) = header.expires {
        let _ = write!(buffer, ",expires={expires}");
    }

    buffer
}
