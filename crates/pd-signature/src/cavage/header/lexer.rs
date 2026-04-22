use logos::{Lexer, Logos, Span};
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

/// Tokenizes the input string into a stream of key-value pairs.
pub fn tokenize(input: &str) -> Tokenizer<'_, impl Iterator<Item = Result<Token, LexError>>> {
    Tokenizer {
        input,
        is_broken: false,
        stream: Token::stream(input),
    }
}

/// Lexical analysis errors for signature header parsing
#[derive(Debug, Diagnostic, Error)]
pub enum LexError {
    /// Encountered an invalid sequence
    #[error("Invalid sequence")]
    InvalidSequence(#[label("Got")] SourceSpan),

    #[error("Unexpected token")]
    UnexpectedToken {
        /// Token type we got
        got: TokenTy,

        // Token type we expected
        expected: TokenTy,

        /// Span of the token
        #[label("Expected: {expected:?}, got: {got:?}")]
        span: SourceSpan,
    },
}

#[derive(Debug, Logos, PartialEq)]
#[logos(skip r"[ \n]+")]
pub enum TokenTy {
    #[regex(r"[a-zA-Z]+")]
    Key,

    #[regex(r"=")]
    Equals,

    // matches either a quoted string or a number
    #[regex(r#""[^"]*"|[0-9]+"#)]
    Value,

    #[regex(r",")]
    Comma,
}

#[derive(Debug)]
pub struct Token {
    pub ty: TokenTy,
    pub span: Span,
}

impl Token {
    pub fn stream(input: &str) -> impl Iterator<Item = Result<Token, LexError>> + '_ {
        Lexer::<TokenTy>::new(input).spanned().map(|(ty, span)| {
            ty.map(|ty| Token {
                ty,
                span: span.clone(),
            })
            .map_err(|()| LexError::InvalidSequence(span.into()))
        })
    }
}

pub struct Tokenizer<'a, I> {
    /// Original input string for span resolution
    input: &'a str,

    /// Stream of tokens from the lexer
    stream: I,

    /// Flag to indicate if we've encountered an error and should stop parsing
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

        if value.ty != $pattern {
            $self.is_broken = true;
            return Some(Err(LexError::UnexpectedToken {
                got: value.ty,
                expected: $pattern,
                span: value.span.into(),
            }));
        }

        value
    }};
}

impl<'a, I> Iterator for Tokenizer<'a, I>
where
    I: Iterator<Item = Result<Token, LexError>>,
{
    type Item = Result<(&'a str, &'a str), LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_broken {
            return None;
        }

        let key = ensure!(self, self.stream.next()?, TokenTy::Key);
        ensure!(self, self.stream.next()?, TokenTy::Equals);
        let value = ensure!(self, self.stream.next()?, TokenTy::Value);

        if let Some(next) = self.stream.next() {
            ensure!(self, next, TokenTy::Comma);
        }

        let key = &self.input[key.span];
        let value = self.input[value.span].trim_matches('"');
        Some(Ok((key, value)))
    }
}
