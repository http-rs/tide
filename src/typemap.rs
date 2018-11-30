use std::sync::RwLock;

use http::Extensions;
use lazy_static::lazy_static;

lazy_static! {
    pub(crate) static ref STATE: RwLock<Extensions> = {
        let m = Extensions::new();
        RwLock::new(m)
    };
}
