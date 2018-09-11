use crate::{
  cached,
  error::*,
  redis::Redis,
  routes::RouteResult,
};

use ffxiv_types::{DataCenter, World, Race, Clan};

use lodestone_scraper::LodestoneScraper;

use lodestone_parser::models::{
  GrandCompany,
  search::{
    Paginated,
    character::CharacterSearchItem,
  },
};

use rocket::State;

use rocket_contrib::Json;

use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
  str::FromStr,
};

#[get("/character/search?<data>")]
crate fn get(data: CharacterSearchData, scraper: State<LodestoneScraper>, redis: Redis) -> Result<Json<RouteResult<Paginated<CharacterSearchItem>>>> {
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
crate struct CharacterSearchData {
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
