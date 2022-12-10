#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("url parse error")]
    UrlParse(#[from] url::ParseError),
    #[error("invalid cookie error")]
    InvalidCookie(String),
    #[error("reqwest error")]
    Reqwest(#[from] reqwest::Error),
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("duplicate error")]
    Duplicate(String),
    #[error("path error")]
    Path(String),
    #[error("parse error")]
    Parse(String),
}
