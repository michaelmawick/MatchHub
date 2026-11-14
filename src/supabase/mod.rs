pub mod oauth;

use std::time::Duration;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::session::Session;

#[derive(Clone)]
pub struct Client {
    url: String,
    publishable_key: String,
    agent: ureq::Agent,
}

pub enum SignUpOutcome {
    SignedIn(Session),
    ConfirmationRequired,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    user: User,
}

#[derive(Deserialize)]
struct User {
    email: Option<String>,
    #[serde(default)]
    user_metadata: Value,
}

#[derive(Deserialize, Default)]
struct ErrorResponse {
    error_code: Option<String>,
    msg: Option<String>,
    error_description: Option<String>,
    message: Option<String>,
}

impl Client {
    pub fn from_env() -> Option<Self> {
        let url = option_env!("SUPABASE_URL")?
            .trim_end_matches('/')
            .to_owned();
        let publishable_key = option_env!("SUPABASE_PUBLISHABLE_KEY")?.to_owned();
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(20)))
            .build()
            .into();
        Some(Self {
            url,
            publishable_key,
            agent,
        })
    }

    pub fn sign_in(&self, email: &str, password: &str) -> Result<Session, String> {
        let body = self.post(
            "/auth/v1/token?grant_type=password",
            json!({ "email": email, "password": password }),
            None,
        )?;
        session_from(body)
    }

    pub fn sign_up(
        &self,
        name: &str,
        email: &str,
        password: &str,
    ) -> Result<SignUpOutcome, String> {
        let body = self.post(
            "/auth/v1/signup",
            json!({ "email": email, "password": password, "data": { "name": name } }),
            None,
        )?;
        if body.get("access_token").is_some() {
            session_from(body).map(SignUpOutcome::SignedIn)
        } else {
            Ok(SignUpOutcome::ConfirmationRequired)
        }
    }

    pub fn refresh(&self, refresh_token: &str) -> Result<Session, String> {
        let body = self.post(
            "/auth/v1/token?grant_type=refresh_token",
            json!({ "refresh_token": refresh_token }),
            None,
        )?;
        session_from(body)
    }

    pub fn exchange_code(&self, code: &str, code_verifier: &str) -> Result<Session, String> {
        let body = self.post(
            "/auth/v1/token?grant_type=pkce",
            json!({ "auth_code": code, "code_verifier": code_verifier }),
            None,
        )?;
        session_from(body)
    }

    pub fn sign_out(&self, access_token: &str) -> Result<(), String> {
        self.post("/auth/v1/logout", json!({}), Some(access_token))
            .map(|_| ())
    }

    pub fn authorize_url(&self, provider: &str, redirect_to: &str, code_challenge: &str) -> String {
        format!(
            "{}/auth/v1/authorize?provider={}&redirect_to={}&code_challenge={}&code_challenge_method=s256",
            self.url,
            percent_encode(provider),
            percent_encode(redirect_to),
            percent_encode(code_challenge),
        )
    }

    fn post(
        &self,
        path: &str,
        payload: Value,
        access_token: Option<&str>,
    ) -> Result<Value, String> {
        let mut request = self
            .agent
            .post(format!("{}{}", self.url, path))
            .header("apikey", &self.publishable_key);
        if let Some(token) = access_token {
            request = request.header("Authorization", format!("Bearer {token}"));
        }

        let mut response = request.send_json(payload).map_err(|_| {
            "Keine Verbindung zum Server. Bitte prüfe deine Internetverbindung.".to_owned()
        })?;

        let status = response.status().as_u16();
        let text = response.body_mut().read_to_string().unwrap_or_default();

        if (200..300).contains(&status) {
            Ok(serde_json::from_str(&text).unwrap_or(Value::Null))
        } else {
            let error = serde_json::from_str::<ErrorResponse>(&text).unwrap_or_default();
            Err(describe_error(status, error))
        }
    }
}

fn session_from(body: Value) -> Result<Session, String> {
    let token: TokenResponse =
        serde_json::from_value(body).map_err(|_| "Unerwartete Antwort vom Server.".to_owned())?;

    let email = token.user.email.unwrap_or_default();
    let name = ["name", "full_name", "user_name"]
        .iter()
        .find_map(|key| token.user.user_metadata.get(key)?.as_str())
        .filter(|name| !name.trim().is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| email.split('@').next().unwrap_or_default().to_owned());

    Ok(Session {
        name,
        email,
        access_token: token.access_token,
        refresh_token: token.refresh_token,
    })
}

fn describe_error(status: u16, error: ErrorResponse) -> String {
    let fallback = error
        .msg
        .or(error.error_description)
        .or(error.message)
        .unwrap_or_else(|| format!("Unbekannter Fehler ({status})."));

    match error.error_code.as_deref() {
        Some("invalid_credentials") => "E-Mail oder Passwort ist falsch.".to_owned(),
        Some("email_not_confirmed") => {
            "Bitte bestätige zuerst deine E-Mail-Adresse über den Link in der E-Mail.".to_owned()
        }
        Some("user_already_exists" | "email_exists") => {
            "Für diese E-Mail-Adresse gibt es bereits ein Konto.".to_owned()
        }
        Some("weak_password") => "Das Passwort ist zu schwach.".to_owned(),
        Some("signup_disabled") => "Die Registrierung ist derzeit deaktiviert.".to_owned(),
        Some("email_address_invalid") => "Diese E-Mail-Adresse wird nicht akzeptiert.".to_owned(),
        Some("over_email_send_rate_limit" | "over_request_rate_limit") => {
            "Zu viele Versuche. Bitte warte einen Moment.".to_owned()
        }
        Some("refresh_token_not_found" | "refresh_token_already_used" | "session_not_found") => {
            "Deine Sitzung ist abgelaufen. Bitte melde dich erneut an.".to_owned()
        }
        Some("flow_state_not_found" | "flow_state_expired" | "bad_code_verifier") => {
            "Die Anmeldung ist abgelaufen. Bitte versuche es erneut.".to_owned()
        }
        _ => fallback,
    }
}

pub fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

pub fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => decoded.push(b' '),
            b'%' if index + 2 < bytes.len() => {
                let high = (bytes[index + 1] as char).to_digit(16);
                let low = (bytes[index + 2] as char).to_digit(16);
                match (high, low) {
                    (Some(high), Some(low)) => {
                        decoded.push((high * 16 + low) as u8);
                        index += 2;
                    }
                    _ => decoded.push(b'%'),
                }
            }
            byte => decoded.push(byte),
        }
        index += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}
