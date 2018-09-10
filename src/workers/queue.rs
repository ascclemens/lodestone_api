use crate::{
  database::{
    models::characters::NewDatabaseCharacter,
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

use r2d2::Pool;

use r2d2_redis::RedisConnectionManager;

use redis::Commands;

crate fn queue(
  redis_pool: &Pool<RedisConnectionManager>,
  db_pool: &Pool<ConnectionManager<PgConnection>>,
) {
  let redis_pool = redis_pool.clone();
  let db_pool = db_pool.clone();
  std::thread::spawn(move || {
    let scraper = LodestoneScraper::default();

    let inner = move || -> Result<()> {
      let redis = redis_pool.get()?;
      let conn = db_pool.get()?;

      let pop: Vec<String> = redis.blpop("character_queue", 0)?;
      let id: u64 = pop[1].parse()?;
      let character = match scraper.character(id) {
        Ok(c) => c,
        Err(lodestone_scraper::error::Error::NotFound) => {
          redis.set_ex(
            &format!("character_{}", id),
            serde_json::to_string(&RouteResult::NotFound::<()>)?,
            1800,
          )?;
          redis.hdel("character_queue_hash", id)?;
          return Ok(());
        },
        Err(e) => {
          redis.hdel("character_queue_hash", id)?;
          return Err(e)?;
        },
      };
      let ndc = NewDatabaseCharacter {
        id: id.into(),
        data: serde_json::to_value(&character)?,
        frecency: crate::frecency::frecency(None),
        last_update: Utc::now().naive_utc(),
      };
      diesel::insert_into(characters::table)
        .values(&ndc)
        .execute(&*conn)?;
      redis.hdel("character_queue_hash", id)?;

      Ok(())
    };
    loop {
      if let Err(e) = inner() {
        eprintln!("error in queue task: {}", e);
      }
      std::thread::sleep(Duration::seconds(5).to_std().unwrap());
    }
  });
}
