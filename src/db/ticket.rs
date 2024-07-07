use std::error::Error as StdError;

use derive_more::Display;
use enum_utils::TryFromRepr;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio_postgres::{
    types::{
        accepts, private::BytesMut, to_sql_checked, FromSql, IsNull, ToSql,
        Type,
    },
    Error,
};
use uuid::Uuid;

use super::{user, Client};

#[derive(Clone, Debug)]
pub struct Ticket {
    pub id: Id,
    pub title: String,
    pub description: String,
    pub status: Status,
    pub count: usize,
    pub price: Option<f64>,
    pub initiator: user::Id,
    pub purchasing_manager: Option<user::Id>,
    pub accounting_manager: Option<user::Id>,
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Display, Serialize)]
pub struct Id(Uuid);

impl Id {
    pub fn new() -> Self {
        Id(Uuid::new_v4())
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
    Clone, Copy, Debug, Deserialize, TryFromRepr, PartialEq, Serialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(u8)]
pub enum Status {
    /// Some materials are requested.
    Requested = 1,

    /// Request is cancelled by the initiator.
    Cancelled = 2,

    /// Manager confirmed that:
    /// - Materials are required by the initiator;
    /// - Materials provider is able to supply them;
    /// - Payment is approved.
    Confirmed = 3,

    /// Manager denied the request because of the following reasons:
    /// - Materials are not required;
    /// - Materials provider is unable to supply them;
    /// - Payment is not approved.
    Denied = 4,

    /// Payment is completed by accounting.
    PaymentCompleted = 5,
}

impl FromSql<'_> for Status {
    accepts!(INT2);

    fn from_sql(
        ty: &Type,
        raw: &[u8],
    ) -> Result<Self, Box<dyn StdError + Sync + Send>> {
        let repr = i16::from_sql(ty, raw)?;
        let repr = u8::try_from(repr)?;
        let role = Self::try_from(repr).map_err(|_| "invalid status")?;
        Ok(role)
    }
}

impl ToSql for Status {
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

impl Client {
    pub async fn get_ticket_by_id(
        &self,
        id: Id,
    ) -> Result<Option<Ticket>, Error> {
        const SQL: &str = "\
            SELECT id, title, description, status, \
                   count, price, initiator_id, \
                   purchasing_manager_id, accounting_manager_id, \
                   created_at \
            FROM tickets \
            WHERE id = $1";
        Ok(self.0.query_opt(SQL, &[&id]).await?.map(|row| Ticket {
            id: row.get("id"),
            title: row.get("title"),
            description: row.get("description"),
            status: row.get("status"),
            count: usize::try_from(row.get::<_, i32>("count")).unwrap(),
            price: row.get("price"),
            initiator: row.get("initiator_id"),
            purchasing_manager: row.get("purchasing_manager_id"),
            accounting_manager: row.get("accounting_manager_id"),
            created_at: row.get("created_at"),
        }))
    }

    pub async fn write_ticket(&self, ticket: &Ticket) -> Result<(), Error> {
        const SQL: &str = "\
            INSERT INTO tickets (id, title, description, status, \
                                 count, price, initiator_id, \
                                 purchasing_manager_id, accounting_manager_id, \
                                 created_at) \
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
            ON CONFLICT (id) DO UPDATE \
            SET title = EXCLUDED.title, \
                description = EXCLUDED.description, \
                status = EXCLUDED.status, \
                count = EXCLUDED.count, \
                price = EXCLUDED.price, \
                initiator_id = EXCLUDED.initiator_id, \
                purchasing_manager_id = EXCLUDED.purchasing_manager_id, \
                accounting_manager_id = EXCLUDED.accounting_manager_id, \
                created_at = EXCLUDED.created_at";

        self.0
            .execute(
                SQL,
                &[
                    &ticket.id,
                    &ticket.title,
                    &ticket.description,
                    &ticket.status,
                    &(ticket.count as i32),
                    &ticket.price,
                    &ticket.initiator,
                    &ticket.purchasing_manager,
                    &ticket.accounting_manager,
                    &ticket.created_at,
                ],
            )
            .await
            .map(drop)
    }

    pub async fn get_tickets_page(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Ticket>, Error> {
        let offset = i64::try_from(offset).unwrap();
        let limit = i64::try_from(limit).unwrap();

        const SQL: &str = "\
            SELECT id, title, description, status, \
                   count, price, initiator_id, \
                   purchasing_manager_id, accounting_manager_id, \
                   created_at \
            FROM tickets \
            ORDER BY created_at DESC, \
                     id DESC \
            OFFSET $1 LIMIT $2";
        Ok(self
            .0
            .query(SQL, &[&offset, &limit])
            .await?
            .into_iter()
            .map(|row| Ticket {
                id: row.get("id"),
                title: row.get("title"),
                description: row.get("description"),
                status: row.get("status"),
                count: usize::try_from(row.get::<_, i32>("count")).unwrap(),
                price: row.get("price"),
                initiator: row.get("initiator_id"),
                purchasing_manager: row.get("purchasing_manager_id"),
                accounting_manager: row.get("accounting_manager_id"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    pub async fn get_tickets_count(&self) -> Result<usize, Error> {
        const SQL: &str = "SELECT COUNT(*) FROM tickets";
        Ok(self
            .0
            .query_one(SQL, &[])
            .await?
            .get::<_, i64>(0)
            .try_into()
            .unwrap())
    }
}
