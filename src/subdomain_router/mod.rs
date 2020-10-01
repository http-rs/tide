use std::collections::BTreeMap;

pub enum SubdomainParams {
    Param(String),
    String(String),
}

pub struct Match<'a, T> {
    pub(crate) data: &'a T,
    pub(crate) params: BTreeMap<&'a String, String>,
}

mod holder;
pub mod router;
