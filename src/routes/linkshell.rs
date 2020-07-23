use crate::{
  cached,
  error::*,
  redis::Redis,
  routes::RouteResult,
};

use lodestone_parser::models::linkshell::Linkshell;

use lodestone_scraper::LodestoneScraper;

use rocket::{State, request::Form};

use rocket_contrib::json::Json;

use tokio::runtime::Runtime;

use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
};

#[get("/linkshell/<id>")]
pub fn get(id: u64, scraper: State<LodestoneScraper>, redis: Redis, runtime: State<Runtime>) -> Result<Json<RouteResult<Linkshell>>> {
  _get(id, LinkshellData { page: 1 }, scraper, redis, runtime)
}

#[get("/linkshell/<id>?<data..>")]
pub fn get_page(id: u64, data: Form<LinkshellData>, scraper: State<LodestoneScraper>, redis: Redis, runtime: State<Runtime>) -> Result<Json<RouteResult<Linkshell>>> {
  _get(id, data.into_inner(), scraper, redis, runtime)
}

#[derive(Debug, FromForm, Hash)]
pub struct LinkshellData {
  page: u64,
}

impl LinkshellData {
  fn as_hash(&self) -> u64 {
    let mut hasher = DefaultHasher::new();
    self.hash(&mut hasher);
    hasher.finish()
  }
}

crate fn _get(id: u64, data: LinkshellData, scraper: State<LodestoneScraper>, mut redis: Redis, runtime: State<Runtime>) -> Result<Json<RouteResult<Linkshell>>> {
  let key = format!("linkshell_{}_{}", id, data.as_hash());
  cached!(runtime, redis, key => {
    runtime.handle().block_on(
      scraper
        .linkshell(id)
        .page(data.page)
        .send()
    ).into()
  })
}
