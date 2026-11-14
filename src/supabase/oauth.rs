use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

use super::{Client, percent_decode};
use crate::session::Session;

pub const CALLBACK_PORT: u16 = 47821;
const CALLBACK_PATH: &str = "/auth/callback";
const TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Clone, Copy)]
pub enum Provider {
    Google,
    Apple,
}

impl Provider {
    pub fn id(self) -> &'static str {
        match self {
            Provider::Google => "google",
            Provider::Apple => "apple",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Provider::Google => "Google",
            Provider::Apple => "Apple",
        }
    }
}

pub fn redirect_url() -> String {
    format!("http://127.0.0.1:{CALLBACK_PORT}{CALLBACK_PATH}")
}

pub fn sign_in(
    client: &Client,
    provider: Provider,
    cancelled: Arc<AtomicBool>,
) -> Result<Session, String> {
    let listener = TcpListener::bind(("127.0.0.1", CALLBACK_PORT)).map_err(|_| {
        format!("Port {CALLBACK_PORT} ist belegt. Bitte schließe andere MatchHub-Fenster und versuche es erneut.")
    })?;
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;

    let code_verifier = random_verifier();
    let code_challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));
    let url = client.authorize_url(provider.id(), &redirect_url(), &code_challenge);

    webbrowser::open(&url).map_err(|_| "Der Browser konnte nicht geöffnet werden.".to_owned())?;

    let started = Instant::now();
    loop {
        if cancelled.load(Ordering::Relaxed) {
            return Err("Anmeldung abgebrochen.".to_owned());
        }
        if started.elapsed() > TIMEOUT {
            return Err(
                "Die Anmeldung hat zu lange gedauert. Bitte versuche es erneut.".to_owned(),
            );
        }

        match listener.accept() {
            Ok((stream, _)) => {
                if let Some(result) = handle_callback(stream) {
                    let code = result?;
                    return client.exchange_code(&code, &code_verifier);
                }
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
}

fn handle_callback(mut stream: TcpStream) -> Option<Result<String, String>> {
    stream.set_nonblocking(false).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok()?;

    let mut buffer = [0u8; 8192];
    let length = stream.read(&mut buffer).ok()?;
    let request = String::from_utf8_lossy(&buffer[..length]);
    let target = request.lines().next()?.split_whitespace().nth(1)?;
    let (path, query) = target.split_once('?').unwrap_or((target, ""));

    if path != CALLBACK_PATH {
        let _ = stream
            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
        return None;
    }

    let parameter = |name: &str| {
        query
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .find(|(key, _)| *key == name)
            .map(|(_, value)| percent_decode(value))
    };

    let result = match (
        parameter("code"),
        parameter("error_description").or_else(|| parameter("error")),
    ) {
        (Some(code), _) => Ok(code),
        (None, Some(error)) => Err(format!("Anmeldung fehlgeschlagen: {error}")),
        (None, None) => Err("Anmeldung fehlgeschlagen: Keine Antwort vom Anbieter.".to_owned()),
    };

    let (title, message) = match &result {
        Ok(_) => (
            "Anmeldung erfolgreich",
            "Du kannst dieses Fenster schließen und zu MatchHub zurückkehren.",
        ),
        Err(_) => (
            "Anmeldung fehlgeschlagen",
            "Bitte kehre zu MatchHub zurück und versuche es erneut.",
        ),
    };
    let html = format!(
        "<!doctype html><html lang=\"de\"><head><meta charset=\"utf-8\"><title>MatchHub</title></head>\
         <body style=\"background:#0e1118;color:#eceff5;font-family:system-ui,sans-serif;display:grid;place-items:center;height:100vh;margin:0\">\
         <div style=\"text-align:center\"><h1>{title}</h1><p style=\"color:#8c94a6\">{message}</p></div></body></html>"
    );
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{html}",
        html.len()
    );
    let _ = stream.write_all(response.as_bytes());

    Some(result)
}

fn random_verifier() -> String {
    let mut bytes = [0u8; 32];
    rand::fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}
