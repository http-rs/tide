use std::collections::HashMap;

use crate::{router::Selection, subdomain::Subdomain};

pub struct Namespace<State> {
    subdomain_map: SubdomainMapper<State>,
}

// enum SubdomainParameter {
//     Param(String),
//     String(String),
// }

struct SubdomainMapper<State> {
    pub mapper: HashMap<String, Subdomain<State>>,
}

impl<State> SubdomainMapper<State> {
    pub fn new() -> Self {
        Self {
            mapper: HashMap::new(),
        }
    }
}

impl<State: Clone + Send + Sync + 'static> Namespace<State> {
    pub fn new() -> Self {
        Self {
            subdomain_map: SubdomainMapper::new(),
        }
    }

    pub fn add(&mut self, subdomain: String, router: Subdomain<State>) -> &mut Subdomain<State> {
        self.subdomain_map
            .mapper
            .entry(subdomain)
            .or_insert_with(|| router)
    }

    /// ```rust
    /// let app = tide::new();
    /// let sub = app.subdomain("api");
    /// sub.with(logging2);
    /// sub.at("/").with(logging).get(Alec)
    /// sub.at("/erik").with(logging).post(Erik)
    /// ```
    pub fn route(
        &self,
        domain: &str,
        path: &str,
        method: http_types::Method,
    ) -> Selection<'_, State> {
        match self.subdomain_map.mapper.get(domain) {
            Some(d) => d.route(path, method),
            None => Selection::not_found_endpoint(),
        }
    }
}
