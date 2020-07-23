use crate::{
  database::{
    models::characters::NewDatabaseCharacter,
    schema::characters,
  },
  error::*,
  routes::RouteResult,
};

use bb8::Pool as AsyncPool;

use chrono::{Duration, Utc};

use diesel::{
  pg::PgConnection,
  prelude::*,
  r2d2::ConnectionManager,
};

use lodestone_scraper::LodestoneScraper;

use r2d2::Pool;

use bb8_redis::{
  redis::AsyncCommands,
  RedisConnectionManager,
};

pub fn queue(
  redis_pool: &AsyncPool<RedisConnectionManager>,
  db_pool: &Pool<ConnectionManager<PgConnection>>,
) {
  let redis_pool = redis_pool.clone();
  let db_pool = db_pool.clone();
  tokio::task::spawn(async move {
    let scraper = LodestoneScraper::default();

    async fn inner(redis_pool: &AsyncPool<RedisConnectionManager>, db_pool: &Pool<ConnectionManager<PgConnection>>, scraper: &LodestoneScraper) -> Result<()> {
      let mut redis = redis_pool.get().await?;
      let conn = db_pool.get()?;

      let pop: Vec<String> = redis.blpop("character_queue", 0).await?;
      let id: u64 = pop[1].parse()?;
      let character = match scraper.character(id).await {
        Ok(c) => c,
        Err(lodestone_scraper::error::Error::NotFound) => {
          redis.set_ex(
            &format!("character_{}", id),
            serde_json::to_string(&RouteResult::NotFound::<()>)?,
            1800,
          ).await?;
          redis.hdel("character_queue_hash", id).await?;
          return Ok(());
        },
        Err(e) => {
          redis.hdel("character_queue_hash", id).await?;
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
      redis.hdel("character_queue_hash", id).await?;

      Ok(())
    };
    loop {
      if let Err(e) = inner(&redis_pool, &db_pool, &scraper).await {
        eprintln!("error in queue task: {}", e);
      }
      tokio::time::delay_for(Duration::seconds(5).to_std().unwrap()).await;
    }
  });
}
