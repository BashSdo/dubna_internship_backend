use serde::{Deserialize, Serialize};

use crate::api;

pub use crate::db::ticket::{Id, Status};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticket {
    pub id: Id,
    pub title: String,
    pub description: String,
    pub status: Status,
    pub count: usize,
    pub price: Option<f64>,
    pub initiator: api::User,
    pub purchasing_manager: Option<api::User>,
    pub accounting_manager: Option<api::User>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct List {
    pub tickets: Vec<Ticket>,
    pub total_count: usize,
}
