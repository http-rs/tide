use std::collections::BTreeMap;

use super::{Match, SubdomainParams};

pub struct Holder<T> {
    data: T,
    map: Vec<SubdomainParams>,
}

impl<T> Holder<T> {
    pub fn new(domain: &str, data: T) -> Holder<T> {
        let map = domain
            .split('.')
            .rev()
            .map(|p| {
                if p.starts_with(":") {
                    SubdomainParams::Param(p[1..].to_owned())
                } else {
                    SubdomainParams::String(p.to_owned())
                }
            })
            .collect();
        Holder { data, map }
    }

    pub fn compare(&self, parts: &Vec<&str>) -> Option<Match<'_, T>> {
        if self.map.len() != parts.len() {
            return None;
        }
        let mut m = Match {
            data: &self.data,
            params: BTreeMap::new(),
        };
        for (url_part, subdomain_part) in parts.iter().zip(&self.map) {
            match subdomain_part {
                SubdomainParams::Param(param_name) => {
                    m.params.insert(param_name, url_part.to_string());
                }
                SubdomainParams::String(exact_name) => {
                    if exact_name == "*" {
                        continue;
                    } else if url_part != exact_name {
                        return None;
                    }
                }
            }
        }
        return Some(m);
    }

    pub fn data(&mut self) -> &mut T {
        &mut self.data
    }
}
