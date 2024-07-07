use std::{error::Error, sync::Arc, time::Duration};

use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, Path, Query, State},
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        request, HeaderValue, Method, StatusCode,
    },
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, RequestPartsExt as _, Router,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use derive_more::From;
use futures::{future::OptionFuture, FutureExt as _};
use itertools::Itertools as _;
use jsonwebtoken::{
    decode, encode, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::{fs, net, task};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{
    layer::SubscriberExt as _, util::SubscriberInitExt as _,
};

use dubna_internship::{api, db, Config};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = fs::read_to_string("config.toml").await?;
    let config = toml::from_str::<Config>(&config)?;

    let (db_client, db_connection) = db::connect(config.db).await?;

    task::spawn(async move {
        if let Err(e) = db_connection.await {
            panic!("database connection failed: {e}");
        }
    });

    let mut cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::PATCH])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE]);
    for origin in &config.http.cors.allowed_origins {
        cors = cors.allow_origin(origin.parse::<HeaderValue>()?);
    }

    let app = Router::new()
        .route("/auth", post(auth))
        .route("/user", get(get_user))
        .route("/ticket", get(list_tickets).post(add_ticket))
        .route("/ticket/:id", get(get_ticket).patch(edit_ticket))
        .layer(cors)
        .with_state(Arc::new(AppState {
            db_client,
            jwt_expiration_time: config.jwt.expiration_time,
            jwt_decoding_key: DecodingKey::from_secret(
                config.jwt.secret.as_bytes(),
            ),
            jwt_encoding_key: EncodingKey::from_secret(
                config.jwt.secret.as_bytes(),
            ),
        }));

    let listener = net::TcpListener::bind(config.http.server.addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Deserialize)]
struct AuthInput {
    login: String,
    password: String,
}

async fn auth(
    State(state): State<SharedAppState>,
    Json(AuthInput { login, password }): Json<AuthInput>,
) -> Result<String, AuthError> {
    use AuthError as E;

    let password_hash = api::user::PasswordHash::new(&password);

    let user = state
        .db_client
        .get_user_by_login(&login)
        .await?
        .filter(|u| u.password_hash == password_hash)
        .ok_or(E::WrongLoginOrPassword)?;

    let expires_at = OffsetDateTime::now_utc() + state.jwt_expiration_time;
    encode(
        &Header::default(),
        &AuthClaims {
            user_id: user.id,
            exp: expires_at.unix_timestamp(),
        },
        &state.jwt_encoding_key,
    )
    .map_err(|_| E::InvalidToken)
}

#[derive(Debug, From)]
pub enum AuthError {
    #[from]
    DbError(db::Error),
    InvalidToken,
    WrongLoginOrPassword,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            Self::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::WrongLoginOrPassword => StatusCode::FORBIDDEN,
        }
        .into_response()
    }
}

async fn get_user(
    State(state): State<SharedAppState>,
    auth_claims: AuthClaims,
) -> Result<Json<api::User>, GetUserError> {
    use GetUserError as E;

    let my = state
        .db_client
        .get_user_by_id(auth_claims.user_id)
        .await?
        .ok_or(E::UserNotFound)?;

    Ok(Json(api::User {
        id: my.id,
        name: my.name,
        role: my.role,
    }))
}

#[derive(Debug, From)]
pub enum GetUserError {
    #[from]
    DbError(db::Error),
    UserNotFound,
}

impl IntoResponse for GetUserError {
    fn into_response(self) -> Response {
        match self {
            Self::DbError(_) | Self::UserNotFound => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
        .into_response()
    }
}

#[derive(Deserialize)]
struct ListTicketsInput {
    offset: usize,
    limit: usize,
}

async fn list_tickets(
    State(state): State<SharedAppState>,
    _: AuthClaims,
    Query(ListTicketsInput { offset, limit }): Query<ListTicketsInput>,
) -> Result<Json<api::ticket::List>, ListTicketsError> {
    use ListTicketsError as E;

    let page_fut = state.db_client.get_tickets_page(offset, limit);
    let total_count_fut = state.db_client.get_tickets_count();
    let (page, total_count) = tokio::try_join!(page_fut, total_count_fut)?;

    let user_ids = page
        .iter()
        .map(|ticket| ticket.initiator)
        .chain(page.iter().filter_map(|ticket| ticket.purchasing_manager))
        .chain(page.iter().filter_map(|ticket| ticket.accounting_manager))
        .unique()
        .collect::<Vec<_>>();
    let users = state.db_client.get_users_by_ids(&user_ids).await?;

    let tickets = page
        .into_iter()
        .map(|ticket| {
            let initiator = users
                .get(&ticket.initiator)
                .ok_or(E::UserNotFound(ticket.initiator))?;
            let purchasing_manager = ticket
                .purchasing_manager
                .map(|id| users.get(&id).ok_or(E::UserNotFound(id)))
                .transpose()?;
            let accounting_manager = ticket
                .accounting_manager
                .map(|id| users.get(&id).ok_or(E::UserNotFound(id)))
                .transpose()?;
            Ok::<_, E>(api::Ticket {
                id: ticket.id,
                title: ticket.title,
                description: ticket.description,
                status: ticket.status,
                count: ticket.count,
                price: ticket.price,
                initiator: api::User {
                    id: initiator.id,
                    name: initiator.name.clone(),
                    role: initiator.role,
                },
                purchasing_manager: purchasing_manager.map(|u| api::User {
                    id: u.id,
                    name: u.name.clone(),
                    role: u.role,
                }),
                accounting_manager: accounting_manager.map(|u| api::User {
                    id: u.id,
                    name: u.name.clone(),
                    role: u.role,
                }),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(api::ticket::List {
        tickets,
        total_count,
    }))
}

#[derive(Debug, From)]
pub enum ListTicketsError {
    #[from]
    DbError(db::Error),
    UserNotFound(api::user::Id),
}

impl IntoResponse for ListTicketsError {
    fn into_response(self) -> Response {
        match self {
            Self::DbError(_) | Self::UserNotFound(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
        .into_response()
    }
}

#[derive(Deserialize)]
struct AddTicketInput {
    title: String,
    description: String,
    count: usize,
}

async fn add_ticket(
    State(state): State<SharedAppState>,
    auth_claims: AuthClaims,
    Json(AddTicketInput {
        title,
        description,
        count,
    }): Json<AddTicketInput>,
) -> Result<Json<api::Ticket>, AddTicketError> {
    use AddTicketError as E;

    let my = state
        .db_client
        .get_user_by_id(auth_claims.user_id)
        .await?
        .ok_or(E::UserNotFound)?;
    if my.role != db::user::Role::Initiator {
        return Err(E::TicketCannotBeCreated);
    }

    let ticket = db::Ticket {
        id: db::ticket::Id::new(),
        title,
        description,
        status: db::ticket::Status::Requested,
        count,
        price: None,
        initiator: my.id,
        purchasing_manager: None,
        accounting_manager: None,
        created_at: OffsetDateTime::now_utc(),
    };

    state.db_client.write_ticket(&ticket).await?;

    Ok(Json(api::Ticket {
        id: ticket.id,
        title: ticket.title,
        description: ticket.description,
        count: ticket.count,
        price: ticket.price,
        initiator: api::User {
            id: my.id,
            name: my.name.clone(),
            role: my.role,
        },
        purchasing_manager: None,
        accounting_manager: None,
        status: ticket.status,
    }))
}

#[derive(Debug, From)]
pub enum AddTicketError {
    #[from]
    DbError(db::Error),
    TicketCannotBeCreated,
    UserNotFound,
}

impl IntoResponse for AddTicketError {
    fn into_response(self) -> Response {
        match self {
            Self::TicketCannotBeCreated => StatusCode::BAD_REQUEST,
            Self::DbError(_) | Self::UserNotFound => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
        .into_response()
    }
}

#[derive(Deserialize)]
#[serde(content = "data", rename_all = "camelCase", tag = "op")]
enum EditTicketInput {
    EditTitle { title: String },
    EditDescription { description: String },
    Cancel,
    Confirm { price: f64 },
    Deny,
    MarkAsPaid,
}

async fn edit_ticket(
    State(state): State<SharedAppState>,
    auth_claims: AuthClaims,
    Path(id): Path<api::ticket::Id>,
    Json(op): Json<EditTicketInput>,
) -> Result<Json<api::Ticket>, EditTicketError> {
    use EditTicketError as E;
    use EditTicketInput as Op;

    let state = &state;

    let my = state
        .db_client
        .get_user_by_id(auth_claims.user_id)
        .await?
        .ok_or(E::UserNotFound)?;
    let mut ticket = state
        .db_client
        .get_ticket_by_id(id)
        .await?
        .ok_or(E::TicketNotFound)?;

    match op {
        Op::EditTitle { title } => {
            if ticket.status != db::ticket::Status::Requested
                || ticket.initiator != my.id
            {
                return Err(E::TicketCannotBeModified);
            }

            ticket.title = title;
        }
        Op::EditDescription { description } => {
            // Description can be used for comments, so should be editable
            // throughout the ticket lifecycle.
            ticket.description = description;
        }
        Op::Cancel => {
            if ticket.status != db::ticket::Status::Requested
                || ticket.initiator != my.id
            {
                return Err(E::TicketCannotBeCancelled);
            }

            ticket.status = db::ticket::Status::Cancelled;
        }
        Op::Confirm { price } => {
            if ticket.status != db::ticket::Status::Requested
                || my.role != db::user::Role::PurchasingManager
            {
                return Err(E::TicketCannotBeConfirmed);
            }

            ticket.status = db::ticket::Status::Confirmed;
            ticket.price = Some(price);
            ticket.purchasing_manager = Some(my.id);
        }
        Op::Deny => {
            if ticket.status != db::ticket::Status::Requested
                || my.role != db::user::Role::PurchasingManager
            {
                return Err(E::TicketCannotBeConfirmed);
            }

            ticket.status = db::ticket::Status::Denied;
            ticket.purchasing_manager = Some(my.id);
        }
        Op::MarkAsPaid => {
            if ticket.status != db::ticket::Status::Confirmed
                || my.role != db::user::Role::AccountingManager
            {
                return Err(E::TicketCannotBePaid);
            }

            ticket.status = db::ticket::Status::PaymentCompleted;
            ticket.accounting_manager = Some(my.id);
        }
    }

    state.db_client.write_ticket(&ticket).await?;

    let initiator = state
        .db_client
        .get_user_by_id(ticket.initiator)
        .await?
        .ok_or(E::UserNotFound)?;
    let purchasing_manager =
        OptionFuture::from(ticket.purchasing_manager.map(|id| async move {
            state
                .db_client
                .get_user_by_id(id)
                .await?
                .ok_or(E::UserNotFound)
        }))
        .map(Option::transpose)
        .await?;
    let accounting_manager =
        OptionFuture::from(ticket.accounting_manager.map(|id| async move {
            state
                .db_client
                .get_user_by_id(id)
                .await?
                .ok_or(E::UserNotFound)
        }))
        .map(Option::transpose)
        .await?;

    Ok(Json(api::Ticket {
        id: ticket.id,
        title: ticket.title,
        description: ticket.description,
        status: ticket.status,
        count: ticket.count,
        price: ticket.price,
        initiator: api::User {
            id: initiator.id,
            name: initiator.name.clone(),
            role: initiator.role,
        },
        purchasing_manager: purchasing_manager.map(|u| api::User {
            id: u.id,
            name: u.name.clone(),
            role: u.role,
        }),
        accounting_manager: accounting_manager.map(|u| api::User {
            id: u.id,
            name: u.name.clone(),
            role: u.role,
        }),
    }))
}

#[derive(Debug, From)]
pub enum EditTicketError {
    #[from]
    DbError(db::Error),
    TicketCannotBeCancelled,
    TicketCannotBeConfirmed,
    TicketCannotBeModified,
    TicketCannotBePaid,
    TicketNotFound,
    UserNotFound,
}

impl IntoResponse for EditTicketError {
    fn into_response(self) -> Response {
        match self {
            Self::TicketCannotBeCancelled
            | Self::TicketCannotBeConfirmed
            | Self::TicketCannotBeModified
            | Self::TicketCannotBePaid => StatusCode::BAD_REQUEST,
            Self::TicketNotFound => StatusCode::NOT_FOUND,
            Self::DbError(_) | Self::UserNotFound => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
        .into_response()
    }
}

async fn get_ticket(
    State(state): State<SharedAppState>,
    _: AuthClaims,
    Path(id): Path<api::ticket::Id>,
) -> Result<Json<api::Ticket>, GetTicketError> {
    use GetTicketError as E;

    let state = &state;

    let ticket = state
        .db_client
        .get_ticket_by_id(id)
        .await?
        .ok_or(E::TicketNotFound)?;

    let initiator = state
        .db_client
        .get_user_by_id(ticket.initiator)
        .await?
        .ok_or(E::UserNotFound)?;
    let purchasing_manager =
        OptionFuture::from(ticket.purchasing_manager.map(|id| async move {
            state
                .db_client
                .get_user_by_id(id)
                .await?
                .ok_or(E::UserNotFound)
        }))
        .map(Option::transpose)
        .await?;
    let accounting_manager =
        OptionFuture::from(ticket.accounting_manager.map(|id| async move {
            state
                .db_client
                .get_user_by_id(id)
                .await?
                .ok_or(E::UserNotFound)
        }))
        .map(Option::transpose)
        .await?;

    Ok(Json(api::Ticket {
        id: ticket.id,
        title: ticket.title,
        description: ticket.description,
        status: ticket.status,
        count: ticket.count,
        price: ticket.price,
        initiator: api::User {
            id: initiator.id,
            name: initiator.name.clone(),
            role: initiator.role,
        },
        purchasing_manager: purchasing_manager.map(|u| api::User {
            id: u.id,
            name: u.name.clone(),
            role: u.role,
        }),
        accounting_manager: accounting_manager.map(|u| api::User {
            id: u.id,
            name: u.name.clone(),
            role: u.role,
        }),
    }))
}

#[derive(Debug, From)]
pub enum GetTicketError {
    #[from]
    DbError(db::Error),
    TicketNotFound,
    UserNotFound,
}

impl IntoResponse for GetTicketError {
    fn into_response(self) -> Response {
        match self {
            Self::TicketNotFound => StatusCode::NOT_FOUND,
            Self::DbError(_) | Self::UserNotFound => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
        .into_response()
    }
}

type SharedAppState = Arc<AppState>;

struct AppState {
    db_client: db::Client,

    jwt_expiration_time: Duration,

    jwt_decoding_key: DecodingKey,

    jwt_encoding_key: EncodingKey,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct AuthClaims {
    user_id: api::user::Id,
    exp: i64,
}

#[async_trait]
impl FromRequestParts<SharedAppState> for AuthClaims {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &SharedAppState,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        let token_data = decode::<Self>(
            bearer.token(),
            &state.jwt_decoding_key,
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}
