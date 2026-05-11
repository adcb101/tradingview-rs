use crate::{
    Result,
    utils::build_request,
};
use serde::{Deserialize, Serialize};

/// Represents a user authorized to access a Pine indicator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationUser {
    pub id: u64,
    pub username: String,
    #[serde(default)]
    pub userpic: String,
    #[serde(default)]
    pub expiration: String,
    #[serde(default)]
    pub created: String,
}

/// Manages permissions (authorized users) for a private Pine Script indicator.
///
/// Allows the indicator owner to list, add, modify, and remove users
/// who are authorized to access the script.
pub struct PinePermManager {
    session_id: String,
    signature: String,
    pine_id: String,
}

impl PinePermManager {
    /// Creates a new `PinePermManager`.
    ///
    /// # Arguments
    /// * `session_id` - The `sessionid` cookie value
    /// * `signature` - The `sessionid_sign` cookie value
    /// * `pine_id` - The indicator ID (e.g. "PUB;XXXXXXXXXXXXXXXXXXXXX")
    pub fn new(session_id: &str, signature: &str, pine_id: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
            signature: signature.to_string(),
            pine_id: pine_id.to_string(),
        }
    }

    fn auth_cookie(&self) -> String {
        format!(
            "sessionid={};sessionid_sign={}",
            self.session_id, self.signature
        )
    }

    fn client(&self) -> Result<reqwest::Client> {
        build_request(Some(&self.auth_cookie()))
    }

    /// Gets the list of users authorized to access the indicator.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of users to fetch (default: 10)
    /// * `order` - Sorting order (e.g. "-created", "user__username", "-expiration,user__username")
    pub async fn get_users(&self, limit: u32, order: &str) -> Result<Vec<AuthorizationUser>> {
        #[derive(Deserialize)]
        struct ListResponse {
            results: Vec<AuthorizationUser>,
        }

        let data: ListResponse = self
            .client()?
            .post(format!(
                "https://www.tradingview.com/pine_perm/list_users/?limit={limit}&order_by={order}"
            ))
            .header("origin", "https://www.tradingview.com")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!(
                "pine_id={}",
                self.pine_id.replace(';', "%3B")
            ))
            .send()
            .await?
            .json()
            .await?;

        Ok(data.results)
    }

    /// Adds a user to the indicator's authorized list.
    ///
    /// # Arguments
    /// * `username` - The TV username to authorize
    /// * `expiration` - Optional expiration date (ISO 8601 format)
    ///
    /// Returns the status string ("ok" or "exists").
    pub async fn add_user(
        &self,
        username: &str,
        expiration: Option<&str>,
    ) -> Result<String> {
        let mut body = format!(
            "pine_id={}&username_recip={}",
            self.pine_id.replace(';', "%3B"),
            username
        );
        if let Some(exp) = expiration {
            body.push_str(&format!("&expiration={exp}"));
        }

        #[derive(Deserialize)]
        struct StatusResponse {
            status: String,
        }

        let data: StatusResponse = self
            .client()?
            .post("https://www.tradingview.com/pine_perm/add/")
            .header("origin", "https://www.tradingview.com")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?
            .json()
            .await?;

        Ok(data.status)
    }

    /// Modifies a user's authorization expiration date.
    ///
    /// # Arguments
    /// * `username` - The TV username
    /// * `expiration` - Optional new expiration date (ISO 8601 format)
    ///
    /// Returns the status string ("ok").
    pub async fn modify_expiration(
        &self,
        username: &str,
        expiration: Option<&str>,
    ) -> Result<String> {
        let mut body = format!(
            "pine_id={}&username_recip={}",
            self.pine_id.replace(';', "%3B"),
            username
        );
        if let Some(exp) = expiration {
            body.push_str(&format!("&expiration={exp}"));
        }

        #[derive(Deserialize)]
        struct StatusResponse {
            status: String,
        }

        let data: StatusResponse = self
            .client()?
            .post("https://www.tradingview.com/pine_perm/modify_user_expiration/")
            .header("origin", "https://www.tradingview.com")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?
            .json()
            .await?;

        Ok(data.status)
    }

    /// Removes a user from the indicator's authorized list.
    ///
    /// # Arguments
    /// * `username` - The TV username to remove
    ///
    /// Returns the status string ("ok").
    pub async fn remove_user(&self, username: &str) -> Result<String> {
        let body = format!(
            "pine_id={}&username_recip={}",
            self.pine_id.replace(';', "%3B"),
            username
        );

        #[derive(Deserialize)]
        struct StatusResponse {
            status: String,
        }

        let data: StatusResponse = self
            .client()?
            .post("https://www.tradingview.com/pine_perm/remove/")
            .header("origin", "https://www.tradingview.com")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?
            .json()
            .await?;

        Ok(data.status)
    }
}
