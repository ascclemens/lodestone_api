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

impl<T, D> From<Result<T, D>> for RouteResult<T>
  where D: Display,
{
  fn from(res: Result<T, D>) -> Self {
    match res {
      Ok(result) => RouteResult::Success { result },
      Err(error) => RouteResult::error(error),
    }
  }
}
