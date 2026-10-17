use serde::Serialize;

use crate::error::{Result, SsaiError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Free,
    Premium,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
    pub region: String,
    pub tier: Tier,
}

impl UserProfile {
    pub fn new(id: &str, name: &str, region: &str, tier: Tier) -> UserProfile {
        UserProfile { id: id.into(), name: name.into(), region: region.into(), tier }
    }
}

pub fn all_users() -> Vec<UserProfile> {
    vec![
        UserProfile::new("alice", "Alice", "us", Tier::Free),
        UserProfile::new("bruno", "Bruno", "br", Tier::Free),
        UserProfile::new("clara", "Clara", "eu", Tier::Free),
        UserProfile::new("diana", "Diana", "us", Tier::Premium),
    ]
}

pub fn find_user(id: &str) -> Result<UserProfile> {
    all_users().into_iter().find(|u| u.id == id).ok_or_else(|| SsaiError::UnknownUser(id.into()))
}
