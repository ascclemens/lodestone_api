#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate failure;
#[macro_use] extern crate serde_derive;

use lodestone_scraper::LodestoneScraper;

use lodestone_parser::models::character::Character;

use rocket_contrib::Json;

mod error;

use crate::error::*;

#[get("/")]
fn index() -> &'static str {
  "Hello, world!"
}

thread_local!(
  static SCRAPER: LodestoneScraper = LodestoneScraper::default();
);

#[get("/character/<id>")]
fn character(id: u64) -> Json<RouteResult<Character>> {
  let rr = match SCRAPER.with(|s| s.character(id)) {
    Ok(result) => RouteResult::Success { result },
    Err(error) => RouteResult::error(error),
  };
  Json(rr)
}

fn main() {
  rocket::ignite()
    .mount("/", routes![
      index,
      character,
    ])
    .launch();
}
