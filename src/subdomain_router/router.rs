use super::{holder::Holder, Match};
use std::collections::BTreeMap;

/// A router made for routing subdomain strings to a resource
pub struct SubdomainRouter<T> {
    subdomains: BTreeMap<(usize, String), Holder<T>>,
}

impl<T> SubdomainRouter<T> {
    pub fn new() -> Self {
        Self {
            subdomains: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, subdomain: &str, element: T) -> &mut T {
        self.subdomains
            .entry((self.subdomains.len(), subdomain.to_owned()))
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
