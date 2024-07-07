pub mod ticket;
pub mod user;

use crate::config;

use tokio_postgres::{tls::NoTlsStream, NoTls, Socket};

pub use tokio_postgres::Error;

pub use self::{ticket::Ticket, user::User};

pub type Connection = tokio_postgres::Connection<Socket, NoTlsStream>;

pub async fn connect(
    config: config::Db,
) -> Result<(Client, Connection), Error> {
    tokio_postgres::connect(&config.url, NoTls)
        .await
        .map(|(client, connection)| (Client(client), connection))
}

pub struct Client(tokio_postgres::Client);
