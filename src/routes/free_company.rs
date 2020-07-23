use crate::{
  cached,
  error::*,
  redis::Redis,
  routes::RouteResult,
};

use lodestone_parser::models::free_company::FreeCompany;

use lodestone_scraper::LodestoneScraper;

use rocket::State;

use rocket_contrib::json::Json;

use tokio::runtime::Runtime;

#[get("/free_company/<id>")]
pub fn get(id: u64, scraper: State<LodestoneScraper>, mut redis: Redis, runtime: State<Runtime>) -> Result<Json<RouteResult<FreeCompany>>> {
  let key = format!("free_company_{}", id);
  cached!(redis, key => {
    runtime.handle().block_on(scraper.free_company(id)).into()
  })
}
