use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub name: String,
    pub is_current: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResurrectableSession {
    pub name: String,
    pub duration: Duration,
}
