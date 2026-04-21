use error_stack::{Report, ResultExt};
use logos::{Lexer, Logos, Span};
use miette::{Diagnostic, SourceSpan};

use crate::cavage::SignatureHeader;

/// Parse a `Signature` HTTP header value into a [`SignatureHeader`] struct
pub fn parse(
    input: &str,
) -> Result<SignatureHeader<'_, impl Iterator<Item = &'_ str>>, Report<ParseError>> {
    let mut tokenizer = Tokenizer {
        inner: Token::parse(input),
        input,
        is_broken: false,
    };

    let mut builder = SignatureHeader::builder();
    while let Some((key, value)) = tokenizer.next().transpose()? {
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
            // Ignore unknown keys as per the spec
            _ => {}
        }
    }

    builder.build().change_context(ParseError::MissingField)
}

/// Signature header parse error
#[derive(Debug, Diagnostic, thiserror::Error)]
pub enum ParseError {
    /// Encountered an invalid sequence
    #[error("Invalid sequence")]
    InvalidSequence(SourceSpan),

    /// Unexpected token
    #[error("Unexpected token")]
    UnexpectedToken {
        /// Token type we got
        got: TokenType,

        // Token type we expected
        expected: TokenType,

        /// Span of the token
        #[label("Expected: {expected:?}, got: {got:?}")]
        span: SourceSpan,
    },

    /// Failed to parse an base 10 integer
    #[error("Radix 10 value parsing failed")]
    Radix10Parse,

    /// Missing field in the header
    #[error("Missing required field")]
    MissingField,
}

struct Tokenizer<'a, I> {
    /// Stream of tokens wrapped into a result
    inner: I,

    /// Reference to the original input
    input: &'a str,

    /// Marker whether we encountered any error
    is_broken: bool,
}

macro_rules! ensure {
    ($self:expr, $value:expr, $pattern:expr) => {{
        let value = match $value {
            Ok(val) => val,
            Err(err) => {
                $self.is_broken = true;
                return Some(Err(err));
            }
        };

        if value.r#type != $pattern {
            $self.is_broken = true;
            return Some(Err(ParseError::UnexpectedToken {
                got: value.r#type,
                expected: $pattern,
                span: value.span.into(),
            }));
        }

        value
    }};
}

impl<'a, I> Iterator for Tokenizer<'a, I>
where
    I: Iterator<Item = Result<Token, ParseError>>,
{
    type Item = Result<(&'a str, &'a str), ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_broken {
            return None;
        }

        let key = ensure!(self, self.inner.next()?, TokenType::Key);
        ensure!(self, self.inner.next()?, TokenType::Equals);
        let value = ensure!(self, self.inner.next()?, TokenType::Value);

        if let Some(next) = self.inner.next() {
            ensure!(self, next, TokenType::Comma);
        }

        let key = &self.input[key.span];
        let value = self.input[value.span].trim_matches('"');
        Some(Ok((key, value)))
    }
}

// e.g. keyId="my-key", signature="abc123", headers="(request-target) date", created=1618884473, expires=1618888073
#[derive(Debug, Logos, PartialEq)]
#[logos(skip r"[ \n]+")]
pub enum TokenType {
    #[regex(r"[a-zA-Z]+")]
    Key,

    #[regex(r"=")]
    Equals,

    #[regex(r#""[^"]*"|[0-9]+"#)]
    Value,

    #[regex(r",")]
    Comma,
}

#[derive(Debug)]
struct Token {
    r#type: TokenType,
    span: Span,
}

impl Token {
    fn parse(input: &str) -> impl Iterator<Item = Result<Token, ParseError>> {
        Lexer::<'_, TokenType>::new(input)
            .spanned()
            .map(|(ty, span)| {
                ty.map({
                    let span = span.clone();
                    |ty| Token { r#type: ty, span }
                })
                .map_err(|()| ParseError::InvalidSequence(span.into()))
            })
    }
}
