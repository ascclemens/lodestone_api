#![feature(plugin, decl_macro, custom_derive)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate serde_derive;

use chrono::{DateTime, Duration, TimeZone, Utc};

use ffxiv_types::{DataCenter, World, Race, Clan};

use lodestone_scraper::LodestoneScraper;

use lodestone_parser::models::{
  GrandCompany,
  character::Character,
  free_company::FreeCompany,
  search::{
    Paginated,
    character::CharacterSearchItem,
    free_company::FreeCompanySearchItem,
  },
};

use r2d2::{Pool, PooledConnection};

use r2d2_redis::RedisConnectionManager;

use redis::Commands;

use rocket::{
  Request, State, Outcome,
  http::Status,
  request::{self, FromRequest},
};

use rocket_contrib::Json;

use serde::{de::DeserializeOwned, Serialize};

use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
  ops::Deref,
  str::FromStr,
};

mod error;

use crate::error::*;

pub type Result<T> = std::result::Result<T, failure::Error>;

macro_rules! cached {
  ($redis:expr, $key:expr => $bl:block) => {{
    if let Some((result, expires)) = find_redis(&$redis, $key.as_str())? {
      return Ok(Json(RouteResult::Cached { result, expires }));
    }
    let res = $bl;
    if let RouteResult::Scraped { result } = res {
      put_redis(&$redis, $key.as_str(), &result)?;
      let expires = Utc.timestamp((Utc::now() + Duration::seconds(3600)).timestamp(), 0);
      return Ok(Json(RouteResult::Cached { result, expires }));
    }
    Ok(Json(res))
  }}
}

fn find_redis<T>(redis: &Redis, key: &str) -> Result<Option<(T, DateTime<Utc>)>>
  where T: DeserializeOwned,
{
  let json: Option<String> = redis.get(key)?;
  match json {
    Some(x) => {
      let json = serde_json::from_str(&x)?;
      let expires_in: i64 = redis::cmd("PTTL").arg(key).query(&***redis)?;
      // we only want second resolution
      let expires = Utc.timestamp((Utc::now() + Duration::milliseconds(expires_in)).timestamp(), 0);
      Ok(Some((json, expires)))
    },
    None => return Ok(None),
  }
}

fn put_redis<T>(redis: &Redis, key: &str, val: T) -> Result<()>
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

#[get("/character/search?<data>")]
fn character_search(data: CharacterSearchData, scraper: State<LodestoneScraper>, redis: Redis) -> Result<Json<RouteResult<Paginated<CharacterSearchItem>>>> {
  let search_key = format!("character_search_{}", data.as_hash());
  cached!(redis, search_key => {
    let mut cs = scraper.character_search();

    if let Some(page) = data.page {
      cs.page(page);
    }

    if let Some(ref name) = data.name {
      cs.name(name);
    }

    if let Some(data_center) = data.data_center {
      if let Ok(dc) = DataCenter::from_str(&data_center) {
        cs.data_center(dc);
      }
    }

    if let Some(world) = data.world {
      if let Ok(w) = World::from_str(&world) {
        cs.world(w);
      }
    }

    if let Some(race) = data.race {
      if let Ok(r) = Race::from_str(&race) {
        cs.race(r);
      }
    }

    if let Some(clan) = data.clan {
      if let Ok(c) = Clan::from_str(&clan) {
        cs.clan(c);
      }
    }

    if let Some(grand_company) = data.grand_company {
      if let Some(gc) = GrandCompany::parse(&grand_company) {
        cs.grand_company(gc);
      }
    }

    cs.send().into()
  })
}

#[derive(Debug, FromForm, Hash)]
struct CharacterSearchData {
  page: Option<u64>,
  name: Option<String>,
  world: Option<String>,
  data_center: Option<String>,
  race: Option<String>,
  clan: Option<String>,
  grand_company: Option<String>,
}

impl CharacterSearchData {
  fn as_hash(&self) -> u64 {
    let mut hasher = DefaultHasher::new();
    self.hash(&mut hasher);
    hasher.finish()
  }
}

#[get("/character/<id>")]
fn character(id: u64, scraper: State<LodestoneScraper>, redis: Redis) -> Result<Json<RouteResult<Character>>> {
  let key = &format!("character_{}", id);
  cached!(redis, key => {
    scraper.character(id).into()
  })
}

#[get("/free_company/search?<data>")]
fn free_company_search(data: FreeCompanySearchData, scraper: State<LodestoneScraper>, redis: Redis) -> Result<Json<RouteResult<Paginated<FreeCompanySearchItem>>>> {
  let key = format!("free_company_search_{}", data.as_hash());
  cached!(redis, key => {
    let mut fcs = scraper.free_company_search();

    if let Some(page) = data.page {
      fcs.page(page);
    }

    if let Some(ref name) = data.name {
      fcs.name(name);
    }

    if let Some(data_center) = data.data_center {
      if let Ok(dc) = DataCenter::from_str(&data_center) {
        fcs.data_center(dc);
      }
    }

    if let Some(world) = data.world {
      if let Ok(w) = World::from_str(&world) {
        fcs.world(w);
      }
    }

    if let Some(grand_company) = data.grand_company {
      if let Some(gc) = GrandCompany::parse(&grand_company) {
        fcs.grand_company(gc);
      }
    }

    fcs.send().into()
  })
}

#[derive(Debug, FromForm, Hash)]
struct FreeCompanySearchData {
  page: Option<u64>,
  name: Option<String>,
  world: Option<String>,
  data_center: Option<String>,
  grand_company: Option<String>,
}

impl FreeCompanySearchData {
  fn as_hash(&self) -> u64 {
    let mut hasher = DefaultHasher::new();
    self.hash(&mut hasher);
    hasher.finish()
  }
}

#[get("/free_company/<id>")]
fn free_company(id: u64, scraper: State<LodestoneScraper>, redis: Redis) -> Result<Json<RouteResult<FreeCompany>>> {
  let key = format!("free_company_{}", id);
  cached!(redis, key => {
    scraper.free_company(id).into()
  })
}

fn main() {
  rocket::ignite()
    .manage(redis_pool())
    .manage(LodestoneScraper::default())
    .mount("/", routes![
      index,
      character,
      character_search,
      free_company,
      free_company_search,
    ])
    .launch();
}

fn redis_pool() -> Pool<RedisConnectionManager> {
  let url = std::env::var("REDIS_URL")
    .expect("missing REDIS_URL environment variable");
  let cm = RedisConnectionManager::new(url.as_str())
    .expect("could not build redis connection manager");
  Pool::builder()
    .build(cm)
    .expect("could not build redis pool")
}

pub struct Redis(pub PooledConnection<RedisConnectionManager>);

impl<'a, 'r> FromRequest<'a, 'r> for Redis {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    let pool = request.guard::<State<Pool<RedisConnectionManager>>>()?;
    match pool.get() {
      Ok(conn) => Outcome::Success(Redis(conn)),
      Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
    }
  }
}

impl Deref for Redis {
  type Target = PooledConnection<RedisConnectionManager>;

  fn deref(&self) -> &Self::Target {
      &self.0
  }
}
