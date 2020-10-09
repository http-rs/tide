use super::{holder::Holder, Match, SubdomainType};
use std::collections::BTreeMap;

/// A router made for routing subdomain strings to a resource
pub struct SubdomainRouter<T> {
    subdomains: BTreeMap<SubdomainType, Holder<T>>,
}

impl<T> SubdomainRouter<T> {
    pub fn new() -> Self {
        Self {
            subdomains: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, subdomain: &str, element: T) -> &mut T {
        let subdomain_type = SubdomainType::new(subdomain);
        self.subdomains
            .entry(subdomain_type)
            .or_insert_with(|| Holder::new(subdomain, element))
            .data()
    }

    pub fn recognize(&self, domain: &str) -> Option<Match<'_, T>> {
        let domain = domain.split('.').rev().collect::<Vec<&str>>();
        for (_, value) in &self.subdomains {
            if let Some(subdomain) = value.compare(&domain) {
                return Some(subdomain);
            }
        }
        None
    }
}
