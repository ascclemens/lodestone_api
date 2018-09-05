use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
  #[fail(display = "scraper error: {}", _0)]
  Scraper(lodestone_scraper::error::Error),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum RouteResult<T> {
  Success {
    result: T,
  },
  Error {
    error: String,
  },
}

impl<T> RouteResult<T> {
  pub fn error<D: Display>(error: D) -> Self {
    RouteResult::Error {
      error: error.to_string(),
    }
  }
}
