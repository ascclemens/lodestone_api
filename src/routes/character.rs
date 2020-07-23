use crate::{
  error::*,
  database::{
    DbConn,
    models::{U64, characters::DatabaseCharacter},
    schema::characters,
  },
  redis::Redis,
  routes::RouteResult,
};

use bb8_redis::redis::AsyncCommands;

use chrono::{TimeZone, Utc};

use diesel::prelude::*;

use lodestone_parser::models::character::Character;

use rocket::State;

use rocket_contrib::json::Json;

use tokio::runtime::Runtime;

#[get("/character/<id>")]
pub fn get(id: u64, conn: DbConn, mut pool: Redis, runtime: State<Runtime>) -> Result<Json<RouteResult<Character>>> {
  // get character stored in database
  let db_char: Option<DatabaseCharacter> = characters::table
    .find(U64(id))
    .get_result(&*conn)
    .optional()?;
  // deserialise and update frecency if character was in database
  if let Some(dbc) = db_char {
    let c: Character = serde_json::from_value(dbc.data)?;
    let new_frecency = crate::frecency::frecency(Some(dbc.frecency));
    diesel::update(characters::table)
      .set(characters::frecency.eq(new_frecency))
      .filter(characters::id.eq(dbc.id))
      .execute(&*conn)?;
    return Ok(Json(RouteResult::Success {
      result: c,
      last_update: Utc.from_utc_datetime(&dbc.last_update),
    }));
  }
  // otherwise find result in redis and return it if present
  if let Ok(Some((rr, _))) = runtime.handle().block_on(crate::find_redis(&mut pool, &format!("character_{}", id))) {
    return Ok(Json(rr));
  }
  let mut redis = runtime.handle().block_on(pool.get())?;
  // if not, add it to the queue
  if let Some(idx) = runtime.handle().block_on(redis.hget("character_queue_hash", id))? {
    return Ok(Json(RouteResult::Adding { queue_position: idx }));
  }
  let pos: u64 = runtime.handle().block_on(redis.rpush("character_queue", id))?;
  runtime.handle().block_on(redis.hset("character_queue_hash", id, pos))?;
  // return position in queue
  Ok(Json(RouteResult::Adding { queue_position: pos }))
}
