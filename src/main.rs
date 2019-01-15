#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use lodestone_scraper::LodestoneScraper;

fn main() {
  let db_pool = lodestone_api::database::pool();
  let redis_pool = lodestone_api::redis::pool();

  lodestone_api::workers::queue(&redis_pool, &db_pool);
  lodestone_api::workers::updater(&db_pool);

  rocket::ignite()
    .manage(db_pool)
    .manage(redis_pool)
    .manage(LodestoneScraper::default())
    .mount("/", routes![
      lodestone_api::routes::index,
      lodestone_api::routes::character::get,
      lodestone_api::routes::search::character::get,
      lodestone_api::routes::free_company::get,
      lodestone_api::routes::search::free_company::get,
      lodestone_api::routes::linkshell::get,
      lodestone_api::routes::linkshell::get_page,
      lodestone_api::routes::search::linkshell::get,
    ])
    .launch();
}
