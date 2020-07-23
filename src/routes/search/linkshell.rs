use crate::{
  error::*,
  redis::Redis,
  routes::RouteResult,
};

use ffxiv_types::{DataCenter, World};

use lodestone_scraper::LodestoneScraper;

use lodestone_parser::models::search::{
  Paginated,
  linkshell::LinkshellSearchItem,
};

use rocket::{State, request::Form};

use rocket_contrib::json::Json;

use tokio::runtime::Runtime;

use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
  str::FromStr,
};

use crate::cached;

#[get("/linkshell/search?<data..>")]
pub fn get(data: Form<LinkshellSearchData>, scraper: State<LodestoneScraper>, mut redis: Redis, runtime: State<Runtime>) -> Result<Json<RouteResult<Paginated<LinkshellSearchItem>>>> {
  let data = data.into_inner();
  let key = format!("linkshell_search_{}", data.as_hash());
  cached!(redis, key => {
    let mut fcs = scraper.linkshell_search();

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

    runtime.handle().block_on(fcs.send()).into()
  })
}

#[derive(Debug, FromForm, Hash)]
pub struct LinkshellSearchData {
  page: Option<u64>,
  name: Option<String>,
  world: Option<String>,
  data_center: Option<String>,
}

impl LinkshellSearchData {
  fn as_hash(&self) -> u64 {
    let mut hasher = DefaultHasher::new();
    self.hash(&mut hasher);
    hasher.finish()
  }
}
