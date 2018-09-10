use chrono::{DateTime, Utc};

use lodestone_parser::error::Error as ParserError;

use lodestone_scraper::error::Error;

use std::fmt::Display;

crate type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status")]
crate enum RouteResult<T> {
  Success {
    result: T,
    last_update: DateTime<Utc>,
  },
  Adding {
    queue_position: u64,
  },
  Scraped {
    result: T,
  },
  Cached {
    result: T,
    expires: DateTime<Utc>,
  },
  NotFound,
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

impl<T> From<std::result::Result<T, Error>> for RouteResult<T> {
  fn from(res: std::result::Result<T, Error>) -> Self {
    match res {
      Ok(result) => RouteResult::Scraped { result },
      Err(Error::NotFound) => RouteResult::NotFound,
      Err(error @ Error::UnexpectedResponse(_)) => RouteResult::error(error),
      Err(Error::Parse(ParserError::InvalidPage(page))) => RouteResult::error(format!(
        "invalid page (1 through {} available)",
        page,
      )),
      Err(e) => {
        eprintln!("error: {:#?}", e);
        RouteResult::error("an internal error occurred. did the lodestone change?")
      },
    }
  }
}
