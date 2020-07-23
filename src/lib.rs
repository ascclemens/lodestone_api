#![feature(decl_macro, proc_macro_hygiene, in_band_lifetimes, crate_visibility_modifier, async_closure)]
#![allow(proc_macro_derive_resolution_fallback, clippy::unreadable_literal)]

#[macro_use] extern crate diesel;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rocket;

use chrono::{DateTime, Duration, TimeZone, Utc};

use r2d2_redis::redis::Commands;

use serde::{de::DeserializeOwned, Serialize};

pub mod database;
mod error;
mod frecency;
pub mod redis;
pub mod routes;
pub mod workers;

use crate::{
  error::*,
  redis::Redis,
};

#[macro_export]
macro_rules! cached {
  ($redis:expr, $key:expr => $bl:block) => {{
    use crate::routes::RouteResult;
    use chrono::{Duration, TimeZone, Utc};
    use rocket_contrib::json::Json;

    if let Some((result, expires)) = crate::find_redis(&mut $redis, $key.as_str())? {
      return Ok(Json(RouteResult::Cached { result, expires }));
    }
    let res = $bl;
    if let RouteResult::Scraped { result } = res {
      crate::put_redis(&mut $redis, $key.as_str(), &result)?;
      let expires = Utc.timestamp((Utc::now() + Duration::seconds(3600)).timestamp(), 0);
      return Ok(Json(RouteResult::Cached { result, expires }));
    }
    Ok(Json(res))
  }}
}

crate fn find_redis<T>(redis: &mut Redis, key: &str) -> Result<Option<(T, DateTime<Utc>)>>
where T: DeserializeOwned,
{
  let json: Option<String> = redis.get(key)?;
  match json {
    Some(x) => {
      let json = serde_json::from_str(&x)?;
      let expires_in: i64 = r2d2_redis::redis::cmd("PTTL").arg(key).query(&mut ***redis)?;
      // we only want second resolution
      let expires = Utc.timestamp((Utc::now() + Duration::milliseconds(expires_in)).timestamp(), 0);
      Ok(Some((json, expires)))
    },
    None => Ok(None),
  }
}

crate fn put_redis<T>(redis: &mut Redis, key: &str, val: T) -> Result<()>
where T: Serialize,
{
  let json = serde_json::to_string(&val)?;
  redis.set_ex(key, json, 3600)?;
  Ok(())
}
