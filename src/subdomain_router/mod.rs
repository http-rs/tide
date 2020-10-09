use std::{cmp::Ordering, collections::BTreeMap};

mod holder;
pub mod router;

pub enum SubdomainParams {
    Param(String),
    String(String),
}

pub struct Match<'a, T> {
    pub(crate) data: &'a T,
    pub(crate) params: BTreeMap<&'a String, String>,
}

#[derive(Eq)]
pub enum SubdomainType {
    Static(String),
    Parametrized(String),
}

impl SubdomainType {
    pub fn new(subdomain: &str) -> SubdomainType {
        let parts = subdomain.split('.').rev();
        for part in parts {
            if part.starts_with(":") {
                return SubdomainType::Parametrized(subdomain.to_owned());
            }
        }
        SubdomainType::Static(subdomain.to_owned())
    }
}

impl Ord for SubdomainType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self {
            SubdomainType::Static(me) => match other {
                SubdomainType::Static(you) => me.cmp(you),
                SubdomainType::Parametrized(_) => Ordering::Less,
            },
            SubdomainType::Parametrized(me) => match other {
                SubdomainType::Static(_) => Ordering::Greater,
                SubdomainType::Parametrized(you) => me.cmp(you),
            },
        }
    }
}

impl PartialOrd for SubdomainType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SubdomainType {
    fn eq(&self, other: &Self) -> bool {
        match self {
            SubdomainType::Static(me) => match other {
                SubdomainType::Static(you) => me == you,
                SubdomainType::Parametrized(_) => false,
            },
            SubdomainType::Parametrized(me) => match other {
                SubdomainType::Static(_) => false,
                SubdomainType::Parametrized(you) => me == you,
            },
        }
    }
}
