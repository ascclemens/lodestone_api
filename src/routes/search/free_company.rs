use crate::{
  error::*,
  redis::Redis,
  routes::RouteResult,
};

use ffxiv_types::{DataCenter, World};

use lodestone_scraper::LodestoneScraper;

use lodestone_parser::models::{
  GrandCompany,
  search::{
    Paginated,
    free_company::FreeCompanySearchItem,
  },
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

#[get("/free_company/search?<data..>")]
pub fn get(data: Form<FreeCompanySearchData>, scraper: State<LodestoneScraper>, mut redis: Redis, runtime: State<Runtime>) -> Result<Json<RouteResult<Paginated<FreeCompanySearchItem>>>> {
  let data = data.into_inner();
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

    runtime.handle().block_on(fcs.send()).into()
  })
}

#[derive(Debug, FromForm, Hash)]
pub struct FreeCompanySearchData {
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
