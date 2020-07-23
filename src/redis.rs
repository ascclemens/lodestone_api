use r2d2::{Pool, PooledConnection};

use r2d2_redis::RedisConnectionManager;

use rocket::{
  Request, State, Outcome,
  http::Status,
  request::{self, FromRequest},
};

use std::ops::{Deref, DerefMut};

pub fn pool() -> Pool<RedisConnectionManager> {
  let url = std::env::var("REDIS_URL")
    .expect("missing REDIS_URL environment variable");
  let cm = RedisConnectionManager::new(url.as_str())
    .expect("could not build redis connection manager");
  Pool::builder()
    .build(cm)
    .expect("could not build redis pool")
}

pub struct Redis(crate PooledConnection<RedisConnectionManager>);

impl<'a, 'r> FromRequest<'a, 'r> for Redis {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    let pool = request.guard::<State<Pool<RedisConnectionManager>>>()?;
    match pool.get() {
      Ok(conn) => Outcome::Success(Redis(conn)),
      Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
    }
  }
}

impl Deref for Redis {
  type Target = PooledConnection<RedisConnectionManager>;

  fn deref(&self) -> &Self::Target {
      &self.0
  }
}

impl DerefMut for Redis {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}
