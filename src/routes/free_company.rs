use crate::{
  cached,
  error::*,
  redis::Redis,
};

use lodestone_parser::models::free_company::FreeCompany;

use lodestone_scraper::LodestoneScraper;

use rocket::State;

use rocket_contrib::Json;

#[get("/free_company/<id>")]
crate fn get(id: u64, scraper: State<LodestoneScraper>, redis: Redis) -> Result<Json<RouteResult<FreeCompany>>> {
  let key = format!("free_company_{}", id);
  cached!(redis, key => {
    scraper.free_company(id).into()
  })
}
