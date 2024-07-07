use serde::{Deserialize, Serialize};

pub use crate::db::user::{Id, PasswordHash, Role};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct User {
    pub id: Id,
    pub name: String,
    pub role: Role,
}
