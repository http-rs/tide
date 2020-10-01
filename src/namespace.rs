use std::{collections::BTreeMap, sync::Arc};

use crate::{
    router::Selection, subdomain::Subdomain, subdomain_router::router::SubdomainRouter, Middleware,
};

pub struct Namespace<State> {
    router: SubdomainRouter<Subdomain<State>>,
}

impl<State: Clone + Send + Sync + 'static> Namespace<State> {
    pub fn new() -> Self {
        Self {
            router: SubdomainRouter::new(),
        }
    }

    pub fn add(&mut self, subdomain: String, router: Subdomain<State>) -> &mut Subdomain<State> {
        self.router.add(&subdomain, router)
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

        match self.router.recognize(&domain) {
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
