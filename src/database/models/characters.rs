use crate::database::{
  models::U64,
  schema::characters,
};

use chrono::NaiveDateTime;

use serde_json::Value;

#[derive(Debug, Queryable, Identifiable, AsChangeset)]
#[table_name = "characters"]
crate struct DatabaseCharacter {
  crate id: U64,
  crate data: Value,
  crate frecency: f64,
  crate last_update: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[table_name = "characters"]
crate struct NewDatabaseCharacter {
  crate id: U64,
  crate data: Value,
  crate frecency: f64,
  crate last_update: NaiveDateTime,
}
