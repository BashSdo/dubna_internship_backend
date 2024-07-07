use constcat::concat;
use dubna_internship::api;
use reqwest::StatusCode;
use serde_json::json;

const BASE_URL: &str = "http://localhost:3000";

pub struct Client {
    inner: reqwest::Client,
    pub auth_token: Option<String>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::new(),
            auth_token: None,
        }
    }

    pub async fn auth(mut self, login: &str, password: &str) -> Self {
        const URL: &str = concat!(BASE_URL, "/auth");

        self.auth_token = Some(
            self.inner
                .post(URL)
                .json(&json!({
                    "login": login,
                    "password": password,
                }))
                .send()
                .await
                .expect("failed to send a request")
                .error_for_status()
                .expect("wrong status code")
                .text()
                .await
                .expect("failed to get a response"),
        );

        self
    }

    pub async fn user(&self) -> Result<api::User, StatusCode> {
        const URL: &str = concat!(BASE_URL, "/user");

        let mut req = self.inner.get(URL);
        if let Some(token) = &self.auth_token {
            req = req.header("Authorization", format!("Bearer {token}"));
        }
        Ok(req
            .send()
            .await
            .expect("failed to send a request")
            .error_for_status()
            .map_err(|e| e.status().expect("status error"))?
            .json::<api::User>()
            .await
            .expect("failed to get a response"))
    }

    pub async fn get_tickets(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<api::ticket::List, StatusCode> {
        const URL: &str = concat!(BASE_URL, "/ticket");

        let mut req = self
            .inner
            .get(format!("{URL}?offset={offset}&limit={limit}"));
        if let Some(token) = &self.auth_token {
            req = req.header("Authorization", format!("Bearer {token}"));
        }
        Ok(req
            .send()
            .await
            .expect("failed to send a request")
            .error_for_status()
            .map_err(|e| e.status().expect("status error"))?
            .json::<api::ticket::List>()
            .await
            .expect("failed to get a response"))
    }

    pub async fn add_ticket(
        &self,
        title: &str,
        description: &str,
        count: usize,
    ) -> Result<api::Ticket, StatusCode> {
        const URL: &str = concat!(BASE_URL, "/ticket");

        let mut req = self.inner.post(URL);
        if let Some(token) = &self.auth_token {
            req = req.header("Authorization", format!("Bearer {token}"));
        }
        Ok(req
            .json(&json!({
                "title": title,
                "description": description,
                "count": count,
            }))
            .send()
            .await
            .expect("failed to send a request")
            .error_for_status()
            .map_err(|e| e.status().expect("status error"))?
            .json::<api::Ticket>()
            .await
            .expect("failed to get a response"))
    }

    pub async fn get_ticket(
        &self,
        id: api::ticket::Id,
    ) -> Result<api::Ticket, StatusCode> {
        const URL: &str = concat!(BASE_URL, "/ticket");

        let mut req = self.inner.get(format!("{URL}/{id}"));
        if let Some(token) = &self.auth_token {
            req = req.header("Authorization", format!("Bearer {token}"));
        }
        Ok(req
            .send()
            .await
            .expect("failed to send a request")
            .error_for_status()
            .map_err(|e| e.status().expect("status error"))?
            .json::<api::Ticket>()
            .await
            .expect("failed to get a response"))
    }

    pub async fn edit_ticket_title(
        &self,
        id: api::ticket::Id,
        title: &str,
    ) -> Result<api::Ticket, StatusCode> {
        const URL: &str = concat!(BASE_URL, "/ticket");

        let mut req = self.inner.patch(format!("{URL}/{id}"));
        if let Some(token) = &self.auth_token {
            req = req.header("Authorization", format!("Bearer {token}"));
        }
        Ok(req
            .json(&json!({
                "op": "editTitle",
                "data": {
                    "title": title,
                }
            }))
            .send()
            .await
            .expect("failed to send a request")
            .error_for_status()
            .map_err(|e| e.status().expect("status error"))?
            .json::<api::Ticket>()
            .await
            .expect("failed to get a response"))
    }

    pub async fn edit_ticket_description(
        &self,
        id: api::ticket::Id,
        description: &str,
    ) -> Result<api::Ticket, StatusCode> {
        const URL: &str = concat!(BASE_URL, "/ticket");

        let mut req = self.inner.patch(format!("{URL}/{id}"));
        if let Some(token) = &self.auth_token {
            req = req.header("Authorization", format!("Bearer {token}"));
        }
        Ok(req
            .json(&json!({
                "op": "editDescription",
                "data": {
                    "description": description,
                }
            }))
            .send()
            .await
            .expect("failed to send a request")
            .error_for_status()
            .map_err(|e| e.status().expect("status error"))?
            .json::<api::Ticket>()
            .await
            .expect("failed to get a response"))
    }

    pub async fn cancel_ticket(
        &self,
        id: api::ticket::Id,
    ) -> Result<api::Ticket, StatusCode> {
        const URL: &str = concat!(BASE_URL, "/ticket");

        let mut req = self.inner.patch(format!("{URL}/{id}"));
        if let Some(token) = &self.auth_token {
            req = req.header("Authorization", format!("Bearer {token}"));
        }
        Ok(req
            .json(&json!({
                "op": "cancel",
            }))
            .send()
            .await
            .expect("failed to send a request")
            .error_for_status()
            .map_err(|e| e.status().expect("status error"))?
            .json::<api::Ticket>()
            .await
            .expect("failed to get a response"))
    }

    pub async fn confirm_ticket(
        &self,
        id: api::ticket::Id,
        price: usize,
    ) -> Result<api::Ticket, StatusCode> {
        const URL: &str = concat!(BASE_URL, "/ticket");

        let mut req = self.inner.patch(format!("{URL}/{id}"));
        if let Some(token) = &self.auth_token {
            req = req.header("Authorization", format!("Bearer {token}"));
        }
        Ok(req
            .json(&json!({
                "op": "confirm",
                "data": {
                    "price": price,
                }
            }))
            .send()
            .await
            .expect("failed to send a request")
            .error_for_status()
            .map_err(|e| e.status().expect("status error"))?
            .json::<api::Ticket>()
            .await
            .expect("failed to get a response"))
    }

    pub async fn deny_ticket(
        &self,
        id: api::ticket::Id,
    ) -> Result<api::Ticket, StatusCode> {
        const URL: &str = concat!(BASE_URL, "/ticket");

        let mut req = self.inner.patch(format!("{URL}/{id}"));
        if let Some(token) = &self.auth_token {
            req = req.header("Authorization", format!("Bearer {token}"));
        }
        Ok(req
            .json(&json!({
                "op": "deny",
            }))
            .send()
            .await
            .expect("failed to send a request")
            .error_for_status()
            .map_err(|e| e.status().expect("status error"))?
            .json::<api::Ticket>()
            .await
            .expect("failed to get a response"))
    }

    pub async fn mark_ticket_as_paid(
        &self,
        id: api::ticket::Id,
    ) -> Result<api::Ticket, StatusCode> {
        const URL: &str = concat!(BASE_URL, "/ticket");

        let mut req = self.inner.patch(format!("{URL}/{id}"));
        if let Some(token) = &self.auth_token {
            req = req.header("Authorization", format!("Bearer {token}"));
        }
        Ok(req
            .json(&json!({
                "op": "markAsPaid",
            }))
            .send()
            .await
            .expect("failed to send a request")
            .error_for_status()
            .map_err(|e| e.status().expect("status error"))?
            .json::<api::Ticket>()
            .await
            .expect("failed to get a response"))
    }
}
