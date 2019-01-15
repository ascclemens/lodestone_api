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

use chrono::{TimeZone, Utc};

use diesel::prelude::*;

use lodestone_parser::models::character::Character;

use redis::Commands;

use rocket_contrib::json::Json;

#[get("/character/<id>")]
pub fn get(id: u64, conn: DbConn, redis: Redis) -> Result<Json<RouteResult<Character>>> {
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
  if let Ok(Some((rr, _))) = crate::find_redis(&redis, &format!("character_{}", id)) {
    return Ok(Json(rr));
  }
  // if not, add it to the queue
  if let Some(idx) = redis.hget("character_queue_hash", id)? {
    return Ok(Json(RouteResult::Adding { queue_position: idx }));
  }
  let pos: u64 = redis.rpush("character_queue", id)?;
  redis.hset("character_queue_hash", id, pos)?;
  // return position in queue
  Ok(Json(RouteResult::Adding { queue_position: pos }))
}
