use crate::{
  database::{
    models::characters::DatabaseCharacter,
    schema::characters,
  },
  error::*,
};

use chrono::{Duration, Utc};

use diesel::{
  pg::PgConnection,
  prelude::*,
  r2d2::ConnectionManager,
};

use lodestone_scraper::LodestoneScraper;

use r2d2::{Pool, PooledConnection};

crate fn updater(db_pool: &Pool<ConnectionManager<PgConnection>>) {
  let db_pool = db_pool.clone();

  std::thread::spawn(move || {
    let scraper = LodestoneScraper::default();

    let update_character = |conn: &PooledConnection<ConnectionManager<PgConnection>>, c: &DatabaseCharacter| -> Result<()> {
      let scraped = scraper.character(*c.id)?;
      let val = serde_json::to_value(&scraped)?;
      diesel::update(characters::table)
        .set((
          characters::last_update.eq(Utc::now().naive_utc()),
          characters::data.eq(val),
        ))
        .filter(characters::id.eq(c.id))
        .execute(&**conn)?;
      std::thread::sleep(Duration::seconds(1).to_std().unwrap());
      Ok(())
    };

    let inner = || -> Result<()> {
      let conn = db_pool.get()?;
      let sql = format!("exp(frecency - (extract(epoch from now()) * {:?}))", crate::frecency::DECAY);
      let twelve_hours_ago = (Utc::now() - Duration::hours(12)).naive_utc();
      let chars: Vec<DatabaseCharacter> = characters::table
        .filter(characters::last_update.lt(twelve_hours_ago))
        .order((
          diesel::dsl::sql::<diesel::sql_types::Float8>(&sql).desc(),
          characters::last_update.asc(),
        ))
        .limit(100)
        .load(&*conn)?;
      for c in chars {
        if let Err(e) = update_character(&conn, &c) {
          eprintln!("error updating character {}: {}", *c.id, e);
        }
      }
      Ok(())
    };
    loop {
      if let Err(e) = inner() {
        eprintln!("error in updater task: {}", e);
      }
      std::thread::sleep(Duration::minutes(1).to_std().unwrap());
    }
  });
}
