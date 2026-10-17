use crate::error::LeaseError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Plan {
    Basic,
    Fan,
    Mega,
}

impl Plan {
    pub fn limit(self) -> usize {
        match self {
            Plan::Basic => 1,
            Plan::Fan => 2,
            Plan::Mega => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Policy {
    RejectNewest,
    KickOldest,
}

impl Policy {
    pub fn as_str(self) -> &'static str {
        match self {
            Policy::RejectNewest => "reject_newest",
            Policy::KickOldest => "kick_oldest",
        }
    }

    pub fn parse(value: &str) -> Result<Policy, LeaseError> {
        match value {
            "reject_newest" => Ok(Policy::RejectNewest),
            "kick_oldest" => Ok(Policy::KickOldest),
            other => Err(LeaseError::InvalidPolicy(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Account {
    pub id: String,
    pub plan: Plan,
    pub limit: usize,
}

impl Account {
    pub fn new(id: &str, plan: Plan) -> Account {
        Account {
            id: id.to_string(),
            plan,
            limit: plan.limit(),
        }
    }
}

pub fn default_accounts() -> Vec<Account> {
    vec![
        Account::new("alice", Plan::Basic),
        Account::new("bob", Plan::Fan),
        Account::new("carol", Plan::Mega),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_limits_match_the_commercial_tiers() {
        assert_eq!(Plan::Basic.limit(), 1);
        assert_eq!(Plan::Fan.limit(), 2);
        assert_eq!(Plan::Mega.limit(), 4);
    }

    #[test]
    fn policy_round_trips_through_its_wire_name() {
        for p in [Policy::RejectNewest, Policy::KickOldest] {
            assert_eq!(Policy::parse(p.as_str()).unwrap(), p);
        }
        assert!(Policy::parse("kick_newest").is_err());
    }
}
