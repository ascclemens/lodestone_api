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

pub fn updater(db_pool: &Pool<ConnectionManager<PgConnection>>) {
  let db_pool = db_pool.clone();

  tokio::task::spawn(async move {
    let scraper = LodestoneScraper::default();

    let prevent_underflow = |conn: &PooledConnection<ConnectionManager<PgConnection>>| -> Result<()> {
      let u_sql = format!("ln(0.001) + (extract(epoch from now()) * {:?})", crate::frecency::DECAY);
      let s_sql = format!("exp(frecency - (extract(epoch from now()) * {:?}))", crate::frecency::DECAY);
      diesel::update(characters::table)
        .set(characters::frecency.eq(diesel::dsl::sql::<diesel::sql_types::Float8>(&u_sql)))
        .filter(characters::frecency.eq(0.0)
          .or(diesel::dsl::sql::<diesel::sql_types::Float8>(&s_sql).lt(0.000001)))
        .execute(&**conn)?;
      Ok(())
    };

    async fn update_character(db_pool: &Pool<ConnectionManager<PgConnection>>, c: &DatabaseCharacter, scraper: &LodestoneScraper) -> Result<()> {
      let scraped = scraper.character(*c.id).await?;
      let conn = db_pool.get()?;
      let val = serde_json::to_value(&scraped)?;
      diesel::update(characters::table)
        .set((
          characters::last_update.eq(Utc::now().naive_utc()),
          characters::data.eq(val),
        ))
        .filter(characters::id.eq(c.id))
        .execute(&conn)?;

      tokio::time::delay_for(Duration::seconds(1).to_std().unwrap()).await;
      Ok(())
    };

    let inner = async || -> Result<()> {
      let conn = db_pool.get()?;
      prevent_underflow(&conn)?;
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
        if let Err(e) = update_character(&db_pool, &c, &scraper).await {
          eprintln!("error updating character {}: {}", *c.id, e);
        }
      }
      Ok(())
    };
    loop {
      if let Err(e) = inner().await {
        eprintln!("error in updater task: {}", e);
      }
      tokio::time::delay_for(Duration::minutes(1).to_std().unwrap()).await;
    }
  });
}
