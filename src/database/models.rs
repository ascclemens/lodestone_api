pub mod characters;

use std::ops::Deref;
use std::error::Error;
use std::fmt::{Display, Formatter, Error as FmtError};

use byteorder::ReadBytesExt;

use diesel::pg::Pg;
use diesel::row::Row;
use diesel::backend::Backend;
use diesel::query_source::Queryable;
use diesel::expression::AsExpression;
use diesel::expression::helper_types::AsExprOf;
use diesel::types::{FromSql, FromSqlRow, HasSqlType};
use diesel::sql_types::BigInt;

#[derive(Debug)]
struct SqlError(String);

impl SqlError {
  fn new<S: AsRef<str>>(s: S) -> Self {
    SqlError(s.as_ref().to_string())
  }
}

impl Display for SqlError {
  fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
    write!(f, "{}", self.0)
  }
}

impl Error for SqlError {
  fn description(&self) -> &str {
    "there was an sql error"
  }

  fn cause(&self) -> Option<&Error> {
    None
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
crate struct U64(crate u64);

impl<DB> Queryable<BigInt, DB> for U64
  where DB: Backend + HasSqlType<BigInt>,
        U64: FromSql<BigInt, DB>
{
  type Row = Self;

  fn build(row: Self::Row) -> Self {
    row
  }
}

impl FromSql<BigInt, Pg> for U64 {
  fn from_sql(bytes: Option<&<Pg as Backend>::RawValue>) -> Result<Self, Box<Error + Send + Sync>> {
    let mut bytes = match bytes {
      Some(b) => b,
      None => return Err(Box::new(SqlError::new("unexpected null")))
    };
    bytes
      .read_u64::<<Pg as Backend>::ByteOrder>()
      .map(U64)
      .map_err(|e| Box::new(e) as Box<Error + Send + Sync>)
  }
}

impl<DB> FromSqlRow<BigInt, DB> for U64
  where DB: Backend + HasSqlType<BigInt>,
        U64: FromSql<BigInt, DB>
{
  fn build_from_row<T: Row<DB>>(row: &mut T) -> Result<Self, Box<Error + Send + Sync>> {
    FromSql::from_sql(row.take())
  }
}

impl<'a> AsExpression<BigInt> for &'a U64 {
  type Expression = AsExprOf<i64, BigInt>;

  fn as_expression(self) -> Self::Expression {
    AsExpression::<BigInt>::as_expression(self.0 as i64)
  }
}

impl AsExpression<BigInt> for U64 {
  type Expression = AsExprOf<i64, BigInt>;

  fn as_expression(self) -> Self::Expression {
    AsExpression::<BigInt>::as_expression(self.0 as i64)
  }
}

impl Deref for U64 {
  type Target = u64;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<u64> for U64 {
  fn from(u: u64) -> Self {
    U64(u)
  }
}

impl From<U64> for u64 {
  fn from(u: U64) -> Self {
    *u
  }
}
