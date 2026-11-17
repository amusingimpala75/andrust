use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub(crate) enum TokenType {
    Char,
    Word,
}

impl TokenType {
    pub(crate) fn parse(s: &str) -> Result<Self, ()> {
        match s {
            "char" => Ok(TokenType::Char),
            "word" => Ok(TokenType::Word),
            _ => Err(()),
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            TokenType::Char => "char",
            TokenType::Word => "word",
        }
    }
}

#[derive(Debug)]
pub(crate) struct Args {
    pub token_type: TokenType,
    pub degree: u8,
    pub files: Vec<PathBuf>,
    pub max_count: u64,
}

#[derive(Debug)]
pub(crate) enum ArgParseError {
    InvalidPath(String),
    NotANumber(String),
    UnrecognizedArg(String),
    InvalidValue(String, Vec<&'static str>),
}

impl Error for ArgParseError {}

impl Display for ArgParseError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            ArgParseError::InvalidPath(s) => write!(f, "Invalid path: {}", s),
            ArgParseError::NotANumber(s) => write!(f, "Not a number: {}", s),
            ArgParseError::UnrecognizedArg(s) => write!(f, "Unrecognized argument: {}", s),
            ArgParseError::InvalidValue(s, options) => {
                write!(
                    f,
                    "Invalid value: {}, should be one of [{}]",
                    s,
                    options.join(", ")
                )
            }
        }
    }
}

impl Args {
    pub(crate) fn parse() -> Result<Args, ArgParseError> {
        let mut result = Args {
            token_type: TokenType::Char,
            degree: 1,
            files: vec![],
            max_count: 256,
        };

        // Iterate over args, skipping first one
        for arg in std::env::args().skip(1) {
            match arg {
                s if s.starts_with("--token-type=") => {
                    result.token_type = TokenType::parse(&s["--token-type=".len()..])
                        .map_err(|_| ArgParseError::InvalidValue(s, vec!["char", "word"]))?;
                }
                s if s.starts_with("--degree=") => {
                    result.degree = s["--degree=".len()..]
                        .parse::<u8>()
                        .map_err(|_| ArgParseError::NotANumber(s))?;
                }
                s if s.starts_with("--max_count=") => {
                    result.max_count = s["--max_count=".len()..]
                        .parse::<u64>()
                        .map_err(|_| ArgParseError::NotANumber(s))?;
                }
                s if s.starts_with("--") => {
                    return Err(ArgParseError::UnrecognizedArg(s));
                }
                // Assume it's a path
                _ => {
                    let path = PathBuf::from(arg.clone());
                    if !path.exists() || !path.is_file() {
                        return Err(ArgParseError::InvalidPath(arg));
                    }
                    result.files.push(path);
                }
            }
        }

        Ok(result)
    }
}
