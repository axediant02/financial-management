use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct SessionStore {
    // token -> expiry
    sessions: Mutex<HashMap<String, Instant>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_session(&self) -> String {
        let token = uuid::Uuid::new_v4().to_string();
        let expiry = Instant::now() + Duration::from_secs(60 * 60 * 12);
        let mut guard = self.sessions.lock().expect("sessions lock poisoned");
        guard.insert(token.clone(), expiry);
        token
    }

    pub fn invalidate(&self, token: &str) {
        let mut guard = self.sessions.lock().expect("sessions lock poisoned");
        guard.remove(token);
    }

    pub fn is_valid(&self, token: &str) -> bool {
        let mut guard = self.sessions.lock().expect("sessions lock poisoned");
        let now = Instant::now();
        guard.retain(|_, expiry| *expiry > now);
        guard.get(token).is_some()
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db_path: std::path::PathBuf,
    pub sessions: std::sync::Arc<SessionStore>,
}

