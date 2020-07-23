use bb8::Pool;

use bb8_redis::RedisConnectionManager;

use rocket::{
  Request, State, Outcome,
  request::{self, FromRequest},
};

use std::ops::Deref;

pub type RedisPool = Pool<RedisConnectionManager>;

pub async fn pool() -> RedisPool {
  let url = std::env::var("REDIS_URL")
    .expect("missing REDIS_URL environment variable");
  let cm = RedisConnectionManager::new(url.as_str())
    .expect("could not build redis connection manager");
  Pool::builder()
    .build(cm)
    .await
    .expect("could not build redis pool")
}

pub struct Redis<'a>(crate State<'a, RedisPool>);

impl<'a, 'r> FromRequest<'a, 'r> for Redis<'r> {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    let pool = request.guard::<State<RedisPool>>()?;
    Outcome::Success(Redis(pool))
  }
}

impl<'a> Deref for Redis<'a> {
  type Target = RedisPool;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
