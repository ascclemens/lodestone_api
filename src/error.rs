use lodestone_parser::error::Error as ParserError;

use lodestone_scraper::error::Error;

use std::fmt::Display;

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

impl<T> From<Result<T, Error>> for RouteResult<T> {
  fn from(res: Result<T, Error>) -> Self {
    match res {
      Ok(result) => RouteResult::Success { result },
      Err(error @ Error::NotFound) => RouteResult::error(error),
      Err(error @ Error::UnexpectedResponse(_)) => RouteResult::error(error),
      Err(Error::Parse(ParserError::InvalidPage(page))) => RouteResult::error(format!(
        "invalid page (1 through {} available)",
        page,
      )),
      Err(_) => RouteResult::error("an internal error occurred. did the Lodestone change?"),
    }
  }
}
