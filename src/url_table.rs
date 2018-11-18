//! Generic types for URL routing.
//!
//! Intended to eventually be pulled into a separate crate.

use std::collections::HashMap;

pub trait Router: Sized {
    type Resource;

    fn subrouter(&self) -> Self;
    fn resource(&self) -> Self::Resource;
    fn table(&self) -> &UrlTable<Self>;
    fn table_mut(&mut self) -> &mut UrlTable<Self>;

    fn route<'a>(&'a self, path: &'a str) -> Option<RouteResult<'a, Self::Resource>> {
        let mut params = Vec::new();
        let mut param_map = HashMap::new();

        let mut table = self.table();

        for segment in path.split('/') {
            if segment.is_empty() {
                continue;
            }

            let UrlTable { next, wildcard, .. } = table;

            if let Some(next_table) = next.get(segment) {
                table = next_table;
            } else if let Some(wildcard) = wildcard {
                params.push(segment);

                if !wildcard.name.is_empty() {
                    param_map.insert(&*wildcard.name, segment);
                }

                table = &wildcard.table;
            } else {
                return None;
            }
        }

        table.accept.as_ref().map(|resource| RouteResult {
            resource,
            route_match: RouteMatch {
                vec: params,
                map: param_map,
            },
        })
    }

    /// Add a new resource at `path`.
    fn at<'a>(&'a mut self, path: &'a str) -> ResourceHandle<'a, Self> {
        self.table_mut().setup(path);
        ResourceHandle { router: self, path }
    }
}

pub struct RouteResult<'a, R> {
    pub resource: &'a R,
    pub route_match: RouteMatch<'a>,
}

pub struct ResourceHandle<'a, R: Router> {
    router: &'a mut R,
    path: &'a str,
}

impl<'a, R: Router> ResourceHandle<'a, R> {
    pub fn nest<F>(self, builder: F)
    where
        F: FnOnce(&mut R),
    {
        let mut subrouter = self.router.subrouter();
        let table = self.router.table_mut().setup(self.path);
        if table.accept.is_some() {
            panic!("This path has a resource");
        }
        if !table.next.is_empty() || table.wildcard.is_some() {
            panic!("This path has child resources");
        }
        builder(&mut subrouter);
        std::mem::swap(table, subrouter.table_mut());
    }
}

impl<'a, R: Router> std::ops::Deref for ResourceHandle<'a, R> {
    type Target = R::Resource;

    fn deref(&self) -> &Self::Target {
        let res = self.router.table().find(self.path);
        if let Some(res) = res {
            res
        } else {
            panic!("The resource for this path has not been initialized");
        }
    }
}

impl<'a, R: Router> std::ops::DerefMut for ResourceHandle<'a, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let new_resource = self.router.resource();
        let table = self.router.table_mut().setup(self.path);
        if table.accept.is_none() {
            table.accept = Some(new_resource);
        }
        table.accept.as_mut().unwrap()
    }
}

/// A generic URL routing table of router `R`.
//
// The implementation uses a very simple-minded tree structure. `UrlTable` is a node,
// with branches corresponding to the next path segment. For concrete segments, the
// `next` table gives the available string matches. For the (at most one) wildcard match,
// the `wildcard` field contains the branch.
//
// If the current URL itself is a route, the `accept` field says what resource it contains.
pub struct UrlTable<R: Router> {
    accept: Option<R::Resource>,
    next: HashMap<String, UrlTable<R>>,
    wildcard: Option<Box<Wildcard<R>>>,
}

struct Wildcard<R: Router> {
    name: String,
    table: UrlTable<R>,
}

/// For a successful match, this structure says how any wildcard components were matched.
///
/// The `vec` field places the matches in the order they appeared in the URL.
/// The `map` component contains any named wildcards (`{foo}`) indexed by name.
pub struct RouteMatch<'a> {
    pub vec: Vec<&'a str>,
    pub map: HashMap<&'a str, &'a str>,
}

impl<R: Router> Default for UrlTable<R> {
    fn default() -> UrlTable<R> {
        Self::new()
    }
}

impl<R: Router> UrlTable<R> {
    /// Create an empty routing table.
    pub fn new() -> UrlTable<R> {
        UrlTable {
            accept: None,
            next: HashMap::new(),
            wildcard: None,
        }
    }

    fn find<'a>(&'a self, path: &'a str) -> Option<&'a R::Resource> {
        let mut table = self;

        for segment in path.split('/') {
            if segment.is_empty() {
                continue;
            }

            let UrlTable { next, wildcard, .. } = table;

            if segment.starts_with('{') && segment.ends_with('}') {
                let wildcard = wildcard.as_ref()?;
                let name = &segment[1..segment.len() - 1];

                match &**wildcard {
                    Wildcard { name: n, .. } if name != n => {
                        panic!("Route {} segment `{{{}}}` conflicts with existing wildcard segment `{{{}}}`", path, name, n);
                    }
                    Wildcard { table: t, .. } => {
                        table = t;
                    }
                }
            } else {
                table = next.get(segment)?;
            }
        }

        table.accept.as_ref()
    }

    fn setup<'a>(&'a mut self, path: &'a str) -> &'a mut UrlTable<R> {
        let mut table = self;

        for segment in path.split('/') {
            if segment.is_empty() {
                continue;
            }

            let UrlTable { next, wildcard, .. } = table;

            if segment.starts_with('{') && segment.ends_with('}') {
                let name = &segment[1..segment.len() - 1];

                if wildcard.is_none() {
                    *wildcard = Some(Box::new(Wildcard {
                        name: name.to_string(),
                        table: UrlTable::new(),
                    }));
                }

                match &mut **wildcard.as_mut().unwrap() {
                    Wildcard { name: n, .. } if name != n => {
                        panic!("Route {} segment `{{{}}}` conflicts with existing wildcard segment `{{{}}}`", path, name, n);
                    }
                    Wildcard { table: t, .. } => {
                        table = t;
                    }
                }
            } else {
                table = next
                    .entry(segment.to_string())
                    .or_insert_with(UrlTable::new);
            }
        }

        table
    }
}

#[derive(Default)]
pub struct GenericRouter<R: Default>(UrlTable<GenericRouter<R>>);

impl<R: Default> Router for GenericRouter<R> {
    type Resource = R;

    fn subrouter(&self) -> GenericRouter<R> {
        GenericRouter::default()
    }

    fn resource(&self) -> R {
        R::default()
    }

    fn table(&self) -> &UrlTable<Self> {
        &self.0
    }

    fn table_mut(&mut self) -> &mut UrlTable<Self> {
        &mut self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_route_no_matches() {
        let table: GenericRouter<()> = GenericRouter::default();

        assert!(table.route("").is_none());
        assert!(table.route("/").is_none());
        assert!(table.route("//").is_none());
        assert!(table.route("foo").is_none());
        assert!(table.route("foo/bar").is_none());
    }

    #[test]
    fn root_matches() {
        let mut table: GenericRouter<()> = GenericRouter::default();
        *table.at("/") = ();

        assert!(table.route("").is_some());
        assert!(table.route("/").is_some());
        assert!(table.route("//").is_some());

        assert!(table.route("foo").is_none());
        assert!(table.route("foo/bar").is_none());
    }

    #[test]
    fn one_fixed_segment_matches() {
        let mut table: GenericRouter<()> = GenericRouter::default();
        *table.at("foo") = ();

        assert!(table.route("").is_none());
        assert!(table.route("/").is_none());
        assert!(table.route("//").is_none());

        assert!(table.route("foo").is_some());
        assert!(table.route("/foo").is_some());
        assert!(table.route("foo/").is_some());
        assert!(table.route("/foo/").is_some());
        assert!(table.route("//foo//").is_some());

        assert!(table.route("foo/bar").is_none());
        assert!(table.route("fo/o").is_none());
    }

    #[test]
    fn multiple_fixed_segment_matches() {
        let mut table: GenericRouter<()> = GenericRouter::default();
        *table.at("foo") = ();
        *table.at("bar") = ();

        assert!(table.route("").is_none());
        assert!(table.route("/").is_none());
        assert!(table.route("//").is_none());

        assert!(table.route("foo").is_some());
        assert!(table.route("bar").is_some());

        assert!(table.route("foo/bar").is_none());
        assert!(table.route("bar/foo").is_none())
    }

    #[test]
    fn nested_fixed_segment_matches() {
        let mut table: GenericRouter<()> = GenericRouter::default();
        *table.at("foo/bar") = ();

        assert!(table.route("").is_none());
        assert!(table.route("foo").is_none());

        assert!(table.route("foo/bar").is_some());
    }

    #[test]
    fn multiple_nested_fixed_segment_matches() {
        let mut table: GenericRouter<()> = GenericRouter::default();
        *table.at("foo/bar") = ();
        *table.at("baz") = ();
        *table.at("quux/twiddle/twibble") = ();

        assert!(table.route("").is_none());
        assert!(table.route("foo").is_none());
        assert!(table.route("quux").is_none());

        assert!(table.route("foo/bar").is_some());
        assert!(table.route("baz").is_some());
        assert!(table.route("quux/twiddle/twibble").is_some());
    }

    #[test]
    fn overlap_nested_fixed_segment_matches() {
        let mut table: GenericRouter<i32> = GenericRouter::default();
        *table.at("") = 0;
        *table.at("foo") = 1;
        *table.at("foo/bar") = 2;

        assert_eq!(*table.route("/").unwrap().resource, 0);
        assert_eq!(*table.route("/foo").unwrap().resource, 1);
        assert_eq!(*table.route("/foo/bar").unwrap().resource, 2);

        assert_eq!(*table.route("").unwrap().resource, 0);
        assert_eq!(*table.route("foo").unwrap().resource, 1);
        assert_eq!(*table.route("foo/bar").unwrap().resource, 2);
    }

    #[test]
    fn wildcard_matches() {
        let mut table: GenericRouter<()> = GenericRouter::default();
        *table.at("{}") = ();

        assert!(table.route("").is_none());
        assert!(table.route("foo/bar").is_none());

        assert!(table.route("foo").is_some());
        assert!(table.route("bar").is_some());
    }

    #[test]
    fn nested_wildcard_matches() {
        let mut table: GenericRouter<()> = GenericRouter::default();
        *table.at("{}/{}") = ();

        assert!(table.route("").is_none());
        assert!(table.route("foo").is_none());

        assert!(table.route("foo/bar").is_some());
        assert_eq!(
            &table.route("foo/bar").unwrap().route_match.vec,
            &["foo", "bar"]
        );
        assert!(table.route("foo/bar").unwrap().route_match.map.is_empty());
    }

    #[test]
    fn mixed_route() {
        let mut table: GenericRouter<()> = GenericRouter::default();
        *table.at("foo/{}/bar") = ();

        assert!(table.route("").is_none());
        assert!(table.route("foo").is_none());
        assert!(table.route("foo/bar").is_none());
        assert!(table.route("foo/bar/baz").is_none());

        assert!(table.route("foo/baz/bar").is_some());
        assert_eq!(
            &table.route("foo/baz/bar").unwrap().route_match.vec,
            &["baz"]
        );
    }

    #[test]
    fn wildcard_fallback() {
        let mut table: GenericRouter<i32> = GenericRouter::default();
        *table.at("foo") = 0;
        *table.at("foo/bar") = 1;
        *table.at("foo/{}/bar") = 2;

        assert!(table.route("").is_none());
        assert!(table.route("foo/bar/baz").is_none());
        assert!(table.route("foo/bar/bar").is_none());

        assert_eq!(*table.route("foo").unwrap().resource, 0);
        assert_eq!(*table.route("foo/bar").unwrap().resource, 1);
        assert_eq!(*table.route("foo/baz/bar").unwrap().resource, 2);
    }
}
