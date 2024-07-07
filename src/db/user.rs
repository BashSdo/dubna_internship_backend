use std::{collections::HashMap, error::Error as StdError};

use enum_utils::TryFromRepr;
use serde::{Deserialize, Serialize};
use tokio_postgres::{
    types::{
        accepts, private::BytesMut, to_sql_checked, FromSql, IsNull, ToSql,
        Type,
    },
    Error,
};
use uuid::Uuid;

use super::Client;

#[derive(Clone, Debug)]
pub struct User {
    pub id: Id,
    pub name: String,
    pub role: Role,
    pub login: String,
    pub password_hash: PasswordHash,
}

#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize,
)]
pub struct Id(Uuid);

impl Id {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<u128> for Id {
    fn from(value: u128) -> Self {
        Self(Uuid::from_u128(value))
    }
}

impl FromSql<'_> for Id {
    accepts!(UUID);

    fn from_sql(
        ty: &Type,
        raw: &[u8],
    ) -> Result<Self, Box<dyn StdError + Sync + Send>> {
        Uuid::from_sql(ty, raw).map(Self)
    }
}

impl ToSql for Id {
    accepts!(UUID);

    to_sql_checked!();

    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn StdError + Sync + Send>> {
        self.0.to_sql(ty, out)
    }
}

#[derive(
    Clone, Copy, Debug, Deserialize, Eq, TryFromRepr, PartialEq, Serialize,
)]
#[repr(u8)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Role {
    Initiator = 1,
    PurchasingManager = 2,
    AccountingManager = 3,
}

impl FromSql<'_> for Role {
    accepts!(INT2);

    fn from_sql(
        ty: &Type,
        raw: &[u8],
    ) -> Result<Self, Box<dyn StdError + Sync + Send>> {
        let repr = i16::from_sql(ty, raw)?;
        let repr = u8::try_from(repr)?;
        let role = Self::try_from(repr).map_err(|_| "invalid role")?;
        Ok(role)
    }
}

impl ToSql for Role {
    accepts!(INT2);

    to_sql_checked!();

    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn StdError + Sync + Send>> {
        let repr = i16::from((*self) as u8);
        repr.to_sql(ty, out)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PasswordHash(String);

impl PasswordHash {
    pub fn new(secret: &str) -> Self {
        // TODO: Use real hash function.
        Self(secret.to_string())
    }
}

impl FromSql<'_> for PasswordHash {
    accepts!(TEXT);

    fn from_sql(
        ty: &Type,
        raw: &[u8],
    ) -> Result<Self, Box<dyn StdError + Sync + Send>> {
        String::from_sql(ty, raw).map(Self)
    }
}

impl ToSql for PasswordHash {
    accepts!(TEXT);

    to_sql_checked!();

    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn StdError + Sync + Send>> {
        self.0.to_sql(ty, out)
    }
}

impl Client {
    pub async fn get_user_by_login(
        &self,
        login: &str,
    ) -> Result<Option<User>, Error> {
        const SQL: &str = "SELECT id, name, login, password_hash, role \
                           FROM users \
                           WHERE login = $1 \
                           LIMIT 1";
        Ok(self.0.query_opt(SQL, &[&login]).await?.map(|row| User {
            id: row.get("id"),
            name: row.get("name"),
            login: row.get("login"),
            password_hash: row.get("password_hash"),
            role: row.get("role"),
        }))
    }

    pub async fn get_user_by_id(&self, id: Id) -> Result<Option<User>, Error> {
        const SQL: &str = "SELECT id, name, login, password_hash, role \
                           FROM users \
                           WHERE id = $1 \
                           LIMIT 1";
        Ok(self.0.query_opt(SQL, &[&id]).await?.map(|row| User {
            id: row.get("id"),
            name: row.get("name"),
            login: row.get("login"),
            password_hash: row.get("password_hash"),
            role: row.get("role"),
        }))
    }

    pub async fn get_users_by_ids(
        &self,
        ids: &[Id],
    ) -> Result<HashMap<Id, User>, Error> {
        const SQL: &str = "SELECT id, name, login, password_hash, role \
                           FROM users \
                           WHERE id IN (SELECT unnest($1::UUID[])) \
                           LIMIT $2";

        let limit = i64::try_from(ids.len()).unwrap();

        Ok(self
            .0
            .query(SQL, &[&ids, &limit])
            .await?
            .into_iter()
            .map(|row| {
                let id = row.get("id");
                let user = User {
                    id,
                    name: row.get("name"),
                    login: row.get("login"),
                    password_hash: row.get("password_hash"),
                    role: row.get("role"),
                };
                (id, user)
            })
            .collect())
    }
}
