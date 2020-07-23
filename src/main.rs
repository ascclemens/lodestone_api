#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use lodestone_scraper::LodestoneScraper;

fn main() {
  let runtime = tokio::runtime::Builder::new()
    .threaded_scheduler()
    .enable_all()
    .build()
    .expect("could not create tokio runtime");

  let db_pool = lodestone_api::database::pool();
  let redis_pool = runtime.handle().block_on(lodestone_api::redis::pool());

  runtime.enter(|| lodestone_api::workers::queue(&redis_pool, &db_pool));
  runtime.enter(|| lodestone_api::workers::updater(&db_pool));

  rocket::ignite()
    .manage(db_pool)
    .manage(redis_pool)
    .manage(LodestoneScraper::default())
    .manage(runtime)
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
