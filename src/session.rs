#[derive(Clone)]
pub struct Session {
    pub name: String,
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
}
