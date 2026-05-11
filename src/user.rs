pub use crate::models::UserCookies;
use crate::{
    Result,
    error::{Error, LoginError},
    utils::build_request,
};
use google_authenticator::{GA_AUTH, get_code};
use reqwest::{Response, header::CONTENT_TYPE};
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, error, info, warn};

impl UserCookies {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
        totp_secret: Option<&str>,
    ) -> Result<Self> {
        let client = build_request(None)?;
        let response = client
            .post("https://www.tradingview.com/accounts/signin/")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(format!(
                "username={username}&password={password}&remember=true"
            ))
            .send()
            .await?;

        let (session, signature, device_token) =
            response
                .cookies()
                .fold((None, None, None), |session_cookies, cookie| {
                    match cookie.name() {
                        "sessionid" => (
                            Some(cookie.value().to_string()),
                            session_cookies.1,
                            session_cookies.2,
                        ),
                        "sessionid_sign" => (
                            session_cookies.0,
                            Some(cookie.value().to_string()),
                            session_cookies.2,
                        ),
                        "device_t" => (
                            session_cookies.0,
                            session_cookies.1,
                            Some(cookie.value().to_string()),
                        ),
                        _ => session_cookies,
                    }
                });
        if session.is_none() || signature.is_none() {
            error!("unable to login, username or password is invalid");
            return Err(Error::Login {
                source: LoginError::InvalidCredentials,
            });
        }

        #[derive(Debug, Deserialize)]
        struct LoginUserResponse {
            user: UserCookies,
        }

        let response: Value = response.json().await?;

        let user: UserCookies;

        if response["error"] == *"" {
            debug!("User data: {:#?}", response);
            warn!("2FA is not enabled for this account");
            info!("User is logged in successfully");
            let login_resp: LoginUserResponse = serde_json::from_value(response)?;

            user = login_resp.user;
        } else if response["error"] == *"2FA_required" {
            if totp_secret.is_none() {
                error!("2FA is enabled for this account, but no TOTP secret was provided");
                return Err(Error::Login {
                    source: LoginError::OTPSecretNotFound,
                });
            }

            let response = Self::handle_mfa(
                totp_secret.unwrap(),
                session.clone().unwrap_or_default().as_str(),
                signature.clone().unwrap_or_default().as_str(),
            )
            .await?;

            let (session, signature, device_token) =
                response
                    .cookies()
                    .fold((None, None, None), |session_cookies, cookie| {
                        match cookie.name() {
                            "sessionid" => (
                                Some(cookie.value().to_string()),
                                session_cookies.1,
                                session_cookies.2,
                            ),
                            "sessionid_sign" => (
                                session_cookies.0,
                                Some(cookie.value().to_string()),
                                session_cookies.2,
                            ),
                            "device_t" => (
                                session_cookies.0,
                                session_cookies.1,
                                Some(cookie.value().to_string()),
                            ),
                            _ => session_cookies,
                        }
                    });

            let body = response.json().await?;
            debug!("2FA login response: {:#?}", body);
            info!("User is logged in successfully");
            let login_resp: LoginUserResponse = serde_json::from_value(body)?;

            user = login_resp.user;

            return Ok(UserCookies {
                session: session.unwrap_or_default(),
                session_signature: signature.unwrap_or_default(),
                device_token: device_token.unwrap_or_default(),
                ..user
            });
        } else {
            error!("unable to login, username or password is invalid");
            return Err(Error::Login {
                source: LoginError::InvalidCredentials,
            });
        }

        Ok(UserCookies {
            session: session.unwrap_or_default(),
            session_signature: signature.unwrap_or_default(),
            device_token: device_token.unwrap_or_default(),
            ..user
        })
    }

    /// Gets user info from session cookies by scraping the TradingView homepage.
    ///
    /// # Arguments
    /// * `session` - The `sessionid` cookie value
    /// * `signature` - The `sessionid_sign` cookie value (optional)
    /// * `location` - The TV regional URL (default: `https://www.tradingview.com/`)
    ///
    /// Returns a `UserCookies` with full profile info.
    pub async fn get_user(
        session: &str,
        signature: &str,
        location: &str,
    ) -> Result<UserCookies> {
        Self::get_user_inner(session, signature, location, 0).await
    }

    async fn get_user_inner(
        session: &str,
        signature: &str,
        location: &str,
        redirect_count: u8,
    ) -> Result<UserCookies> {
        if redirect_count > 5 {
            return Err(Error::Login {
                source: LoginError::InvalidSession,
            });
        }

        let cookie = if signature.is_empty() {
            format!("sessionid={session}")
        } else {
            format!("sessionid={session};sessionid_sign={signature}")
        };

        let client = build_request(Some(&cookie))?;
        let response = client
            .get(location)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .send()
            .await?;

        let redirect_location = response
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let data: String = response.text().await?;

        if data.contains("auth_token") {
            let re = regex::Regex::new(r#""id":([0-9]{1,10})"#).unwrap();
            let id: u32 = re
                .captures(&data)
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);

            let re = regex::Regex::new(r#""username":"(.*?)""#).unwrap();
            let username = re
                .captures(&data)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            let re = regex::Regex::new(r#""first_name":"(.*?)""#).unwrap();
            let first_name = re
                .captures(&data)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            let re = regex::Regex::new(r#""last_name":"(.*?)""#).unwrap();
            let last_name = re
                .captures(&data)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            let re = regex::Regex::new(r#""reputation":([0-9.]+)"#).unwrap();
            let reputation: f64 = re
                .captures(&data)
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0.0);

            let re = regex::Regex::new(r#""following":([0-9]+)"#).unwrap();
            let following: u32 = re
                .captures(&data)
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);

            let re = regex::Regex::new(r#""followers":([0-9]+)"#).unwrap();
            let followers: u32 = re
                .captures(&data)
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);

            let re =
                regex::Regex::new(r#""notification_count":\{"following":([0-9]*),"user":([0-9]*)}"#)
                    .unwrap();
            let (notif_following, notif_user) = re
                .captures(&data)
                .map(|c| {
                    let f = c.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
                    let u = c.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
                    (f, u)
                })
                .unwrap_or((0, 0));

            let re = regex::Regex::new(r#""session_hash":"(.*?)""#).unwrap();
            let session_hash = re
                .captures(&data)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            let re = regex::Regex::new(r#""private_channel":"(.*?)""#).unwrap();
            let private_channel = re
                .captures(&data)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            let re = regex::Regex::new(r#""auth_token":"(.*?)""#).unwrap();
            let auth_token = re
                .captures(&data)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            let re = regex::Regex::new(r#""date_joined":"(.*?)""#).unwrap();
            let join_date = re
                .captures(&data)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            Ok(UserCookies {
                id,
                username,
                first_name,
                last_name,
                reputation,
                following,
                followers,
                notifications: crate::models::UserNotifications {
                    user: notif_user,
                    following: notif_following,
                },
                session: session.to_string(),
                session_signature: signature.to_string(),
                session_hash,
                private_channel,
                auth_token,
                join_date,
                ..Default::default()
            })
        } else if let Some(new_location) = redirect_location {
            if new_location != location {
                return Box::pin(Self::get_user_inner(
                    session,
                    signature,
                    &new_location,
                    redirect_count + 1,
                ))
                .await;
            }
            Err(Error::Login {
                source: LoginError::InvalidSession,
            })
        } else {
            Err(Error::Login {
                source: LoginError::InvalidSession,
            })
        }
    }

    async fn handle_mfa(totp_secret: &str, session: &str, signature: &str) -> Result<Response> {
        if totp_secret.is_empty() {
            return Err(Error::Login {
                source: LoginError::OTPSecretNotFound,
            });
        }

        let client = build_request(Some(&format!(
            "sessionid={session}; sessionid_sign={signature};"
        )))?;

        let response = client
            .post("https://www.tradingview.com/accounts/two-factor/signin/totp/")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(format!(
                "code={}",
                match get_code!(totp_secret) {
                    Ok(code) => code,
                    Err(e) => {
                        error!("Error generating TOTP code: {}", e);
                        return Err(Error::Login {
                            source: LoginError::InvalidOTPSecret,
                        });
                    }
                }
            ))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response)
        } else {
            Err(Error::Login {
                source: LoginError::InvalidOTPSecret,
            })
        }
    }
}
