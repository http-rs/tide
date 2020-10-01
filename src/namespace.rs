use std::{collections::BTreeMap, collections::HashMap, sync::Arc};

use crate::{router::Selection, subdomain::Subdomain, Middleware};

enum Param {
    Param(String),
    String(String),
}

struct Holder<T> {
    data: T,
    map: Vec<Param>,
}

struct Match<'a, T> {
    data: &'a T,
    params: BTreeMap<&'a String, String>,
}

impl<T> Holder<T> {
    pub fn new(domain: &str, data: T) -> Holder<T> {
        let map = domain
            .split('.')
            .rev()
            .map(|p| {
                if p.starts_with(":") {
                    Param::Param(p[1..].to_owned())
                } else {
                    Param::String(p.to_owned())
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
                Param::Param(param_name) => {
                    m.params.insert(param_name, url_part.to_string());
                }
                Param::String(exact_name) => {
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

pub struct Namespace<State> {
    subdomain_map: SubdomainMapper<Subdomain<State>>,
}

struct SubdomainMapper<T> {
    pub subdomains: HashMap<String, Holder<T>>,
}

impl<T> SubdomainMapper<T> {
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

impl<State: Clone + Send + Sync + 'static> Namespace<State> {
    pub fn new() -> Self {
        Self {
            subdomain_map: SubdomainMapper::new(),
        }
    }

    pub fn add(&mut self, subdomain: String, router: Subdomain<State>) -> &mut Subdomain<State> {
        self.subdomain_map.add(&subdomain, router)
    }

    pub fn route(
        &self,
        domain: &str,
        path: &str,
        method: http_types::Method,
        global_middleware: &[Arc<dyn Middleware<State>>],
    ) -> NamespaceSelection<'_, State> {
        let subdomains = domain.split('.').rev().skip(2).collect::<Vec<&str>>();
        let domain = if subdomains.len() == 0 {
            "".to_owned()
        } else {
            subdomains
                .iter()
                .rev()
                .fold(String::new(), |sub, part| sub + "." + part)[1..]
                .to_owned()
        };

        match self.subdomain_map.recognize(&domain) {
            Some(data) => {
                let subdomain = data.data;
                let params = data.params;
                let selection = subdomain.route(path, method);
                let subdomain_middleware = subdomain.middleware().as_slice();
                let global_middleware = global_middleware;
                let mut middleware = vec![];
                middleware.extend_from_slice(global_middleware);
                middleware.extend_from_slice(subdomain_middleware);
                NamespaceSelection {
                    selection,
                    middleware,
                    params,
                }
            }
            None => {
                let selection = Selection::not_found_endpoint();
                let mut middleware = vec![];
                middleware.extend_from_slice(global_middleware);
                NamespaceSelection {
                    selection,
                    middleware,
                    params: BTreeMap::new(),
                }
            }
        }
    }
}

pub struct NamespaceSelection<'a, State> {
    pub(crate) selection: Selection<'a, State>,
    pub(crate) middleware: Vec<Arc<dyn Middleware<State>>>,
    pub(crate) params: BTreeMap<&'a String, String>,
}

impl<'a, State> NamespaceSelection<'a, State> {
    pub fn subdomain_params(&self) -> route_recognizer::Params {
        let mut params = route_recognizer::Params::new();
        for (key, value) in &self.params {
            params.insert(key.to_string(), value.to_owned());
        }
        params
    }
}
