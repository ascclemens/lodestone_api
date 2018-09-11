use chrono::{DateTime, Utc};

use lodestone_parser::error::Error as ParserError;

use lodestone_scraper::error::Error;

use std::fmt::Display;

pub mod character;
pub mod free_company;

pub mod search;

#[get("/")]
crate fn index() -> &'static str {
  "Hello, world!"
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum RouteResult<T> {
  /// The resource was successfully retrieved from the database.
  Success {
    /// The resource
    result: T,
    /// The date at which the resource was last scraped and updated
    last_update: DateTime<Utc>,
  },
  /// The resource wasn't found, so it has been queued for scraping.
  Adding {
    /// The position the resource is in its scrape queue
    queue_position: u64,
  },
  /// The resource was scraped once and returned.
  Scraped {
    /// The resource
    result: T,
  },
  /// The resource was scraped and cached for a limited amount of time.
  Cached {
    /// The resource
    result: T,
    /// When the resource will expire from the cache, after which new requests will result in a new
    /// scrape
    expires: DateTime<Utc>,
  },
  /// The resource was not found.
  NotFound,
  /// An error ocurred when processing the route.
  Error {
    /// The error message
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
