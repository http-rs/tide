use super::{holder::domain_type, holder::Holder, Match, SubdomainType};
use std::collections::BTreeMap;

/// A router made for routing subdomain strings to a resource
pub struct SubdomainRouter<T> {
    static_subdomains: BTreeMap<String, Holder<T>>,
    parameterized_subdomains: BTreeMap<String, Holder<T>>,
}

impl<T> SubdomainRouter<T> {
    pub fn new() -> Self {
        Self {
            static_subdomains: BTreeMap::new(),
            parameterized_subdomains: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, subdomain: &str, element: T) -> &mut T {
        let list = match domain_type(subdomain) {
            SubdomainType::Static => &mut self.static_subdomains,
            SubdomainType::Parametrized => &mut self.parameterized_subdomains,
        };
        list.entry(subdomain.to_owned())
            .or_insert_with(|| Holder::new(subdomain, element))
            .data()
    }

    pub fn recognize(&self, domain: &str) -> Option<Match<'_, T>> {
        let domain = domain.split('.').rev().collect::<Vec<&str>>();
        for (_, value) in &self.static_subdomains {
            if let Some(subdomain) = value.compare(&domain) {
                return Some(subdomain);
            }
        }
        for (_, value) in &self.parameterized_subdomains {
            if let Some(subdomain) = value.compare(&domain) {
                return Some(subdomain);
            }
        }
        None
    }
}
