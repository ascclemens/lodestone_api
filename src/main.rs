#![feature(plugin, decl_macro, custom_derive)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate serde_derive;

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

use rocket_contrib::Json;

use std::str::FromStr;

mod error;

use crate::error::*;

thread_local!(
  static SCRAPER: LodestoneScraper = LodestoneScraper::default();
);

#[get("/")]
fn index() -> &'static str {
  "Hello, world!"
}

#[get("/character/search?<data>")]
fn character_search(data: CharacterSearchData) -> Json<RouteResult<Paginated<CharacterSearchItem>>> {
  Json(SCRAPER.with(|s| {
    let mut cs = s.character_search();

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

    cs.send()
  }).into())
}

#[derive(Debug, FromForm)]
struct CharacterSearchData {
  page: Option<u64>,
  name: Option<String>,
  world: Option<String>,
  data_center: Option<String>,
  race: Option<String>,
  clan: Option<String>,
  grand_company: Option<String>,
}

#[get("/character/<id>")]
fn character(id: u64) -> Json<RouteResult<Character>> {
  Json(SCRAPER.with(|s| s.character(id)).into())
}

#[get("/free_company/search?<data>")]
fn free_company_search(data: FreeCompanySearchData) -> Json<RouteResult<Paginated<FreeCompanySearchItem>>> {
  Json(SCRAPER.with(|s| {
    let mut fcs = s.free_company_search();

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

    fcs.send()
  }).into())
}

#[derive(Debug, FromForm)]
struct FreeCompanySearchData {
  page: Option<u64>,
  name: Option<String>,
  world: Option<String>,
  data_center: Option<String>,
  grand_company: Option<String>,
}

#[get("/free_company/<id>")]
fn free_company(id: u64) -> Json<RouteResult<FreeCompany>> {
  Json(SCRAPER.with(|s| s.free_company(id)).into())
}

fn main() {
  rocket::ignite()
    .mount("/", routes![
      index,
      character,
      character_search,
      free_company,
      free_company_search,
    ])
    .launch();
}
