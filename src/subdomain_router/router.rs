use std::collections::HashMap;

use super::{holder::Holder, Match};

pub struct SubdomainRouter<T> {
    subdomains: HashMap<String, Holder<T>>,
}

impl<T> SubdomainRouter<T> {
    pub fn new() -> Self {
        Self {
            subdomains: HashMap::new(),
        }
    }

    pub fn add(&mut self, subdomain: &str, element: T) -> &mut T {
        self.subdomains
            .entry(subdomain.to_owned())
            .or_insert_with(|| Holder::new(subdomain, element))
            .data()
    }

    pub fn recognize(&self, domain: &str) -> Option<Match<'_, T>> {
        let domain = domain.split('.').rev().collect::<Vec<&str>>();
        for value in self.subdomains.values() {
            if let Some(subdomain) = value.compare(&domain) {
                return Some(subdomain);
            }
        }
        None
    }
}
