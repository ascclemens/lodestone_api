#![feature(plugin, decl_macro, custom_derive, in_band_lifetimes)]
#![plugin(rocket_codegen)]
#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use] extern crate diesel;
#[macro_use] extern crate serde_derive;

use chrono::{DateTime, Duration, TimeZone, Utc};

use lodestone_scraper::LodestoneScraper;

use redis::Commands;

use serde::{de::DeserializeOwned, Serialize};

mod database;
mod error;
mod frecency;
mod redis;
mod routes;
mod workers;

use crate::{
  error::*,
  redis::Redis,
};

#[macro_export]
macro_rules! cached {
  ($redis:expr, $key:expr => $bl:block) => {{
    use crate::error::RouteResult;
    use chrono::{Duration, TimeZone, Utc};
    use rocket_contrib::Json;

    if let Some((result, expires)) = crate::find_redis(&$redis, $key.as_str())? {
      return Ok(Json(RouteResult::Cached { result, expires }));
    }
    let res = $bl;
    if let RouteResult::Scraped { result } = res {
      crate::put_redis(&$redis, $key.as_str(), &result)?;
      let expires = Utc.timestamp((Utc::now() + Duration::seconds(3600)).timestamp(), 0);
      return Ok(Json(RouteResult::Cached { result, expires }));
    }
    Ok(Json(res))
  }}
}

crate fn find_redis<T>(redis: &Redis, key: &str) -> Result<Option<(T, DateTime<Utc>)>>
  where T: DeserializeOwned,
{
  let json: Option<String> = redis.get(key)?;
  match json {
    Some(x) => {
      let json = serde_json::from_str(&x)?;
      let expires_in: i64 = ::redis::cmd("PTTL").arg(key).query(&***redis)?;
      // we only want second resolution
      let expires = Utc.timestamp((Utc::now() + Duration::milliseconds(expires_in)).timestamp(), 0);
      Ok(Some((json, expires)))
    },
    None => return Ok(None),
  }
}

crate fn put_redis<T>(redis: &Redis, key: &str, val: T) -> Result<()>
  where T: Serialize,
{
  let json = serde_json::to_string(&val)?;
  redis.set_ex(key, json, 3600)?;
  Ok(())
}

#[get("/")]
fn index() -> &'static str {
  "Hello, world!"
}

fn main() {
  let db_pool = crate::database::pool();
  let redis_pool = crate::redis::pool();

  crate::workers::queue(&redis_pool, &db_pool);
  crate::workers::updater(&db_pool);

  rocket::ignite()
    .manage(db_pool)
    .manage(redis_pool)
    .manage(LodestoneScraper::default())
    .mount("/", routes![
      index,
      crate::routes::character::get,
      crate::routes::search::character::get,
      crate::routes::free_company::get,
      crate::routes::search::free_company::get,
    ])
    .launch();
}
