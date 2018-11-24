//! Generic types for URL routing.
//!
//! Intended to eventually be pulled into a separate crate.

use std::collections::HashMap;

/// A generic URL routing table, terminating with resources `R`.
//
// The implementation uses a very simple-minded tree structure. `UrlTable` is a node,
// with branches corresponding to the next path segment. For concrete segments, the
// `next` table gives the available string matches. For the (at most one) wildcard match,
// the `wildcard` field contains the branch.
//
// If the current URL itself is a route, the `accept` field says what resource it contains.
pub struct UrlTable<R> {
    accept: Option<R>,
    next: HashMap<String, UrlTable<R>>,
    wildcard: Option<Box<Wildcard<R>>>,
}

struct Wildcard<R> {
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

impl<R> std::fmt::Debug for UrlTable<R> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        struct Children<'a, R>(&'a HashMap<String, UrlTable<R>>, Option<&'a Wildcard<R>>);
        impl<'a, R> std::fmt::Debug for Children<'a, R> {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                let mut dbg = fmt.debug_map();
                dbg.entries(self.0.iter());
                if let Some(wildcard) = self.1 {
                    dbg.entry(&format!("{{{}}}", wildcard.name), &wildcard.table);
                }
                dbg.finish()
            }
        }

        fmt.debug_struct("UrlTable")
            .field(
                "resource",
                &format_args!(
                    "{}",
                    if self.accept.is_some() {
                        "Some"
                    } else {
                        "None"
                    }
                ),
            )
            .field(
                "children",
                &Children(&self.next, self.wildcard.as_ref().map(|x| &**x)),
            )
            .finish()
    }
}

impl<R> UrlTable<R> {
    /// Create an empty routing table.
    pub fn new() -> UrlTable<R> {
        UrlTable {
            accept: None,
            next: HashMap::new(),
            wildcard: None,
        }
    }

    /// Get the resource of current path.
    #[allow(dead_code)]
    pub fn resource(&self) -> Option<&R> {
        self.accept.as_ref()
    }

    /// Retrieve a mutable reference of the resource.
    pub fn resource_mut(&mut self) -> &mut Option<R> {
        &mut self.accept
    }

    /// Return an iterator of all resources.
    #[allow(dead_code)]
    pub fn resources(&self) -> Resources<R> {
        Resources { stack: vec![self] }
    }

    /// Return an iterator of mutable references all resources.
    pub fn resources_mut(&mut self) -> ResourcesMut<R> {
        ResourcesMut { stack: vec![self] }
    }

    /// Determine which resource, if any, the conrete `path` should be routed to.
    pub fn route<'a>(&'a self, path: &'a str) -> Option<(&'a R, RouteMatch<'a>)> {
        let mut table = self;
        let mut params = Vec::new();
        let mut param_map = HashMap::new();

        for segment in path.split('/') {
            if segment.is_empty() {
                continue;
            }

            if let Some(next_table) = table.next.get(segment) {
                table = next_table;
            } else if let Some(wildcard) = &table.wildcard {
                params.push(segment);

                if !wildcard.name.is_empty() {
                    param_map.insert(&*wildcard.name, segment);
                }

                table = &wildcard.table;
            } else {
                return None;
            }
        }

        table.accept.as_ref().map(|res| {
            (
                res,
                RouteMatch {
                    vec: params,
                    map: param_map,
                },
            )
        })
    }

    fn wildcard_mut(&mut self) -> Option<&mut Wildcard<R>> {
        self.wildcard.as_mut().map(|b| &mut **b)
    }

    /// Return the table of the given routing path (which may contain wildcards).
    ///
    /// If it doesn't already exist, this will make a new one.
    pub fn setup_table(&mut self, path: &str) -> &mut UrlTable<R> {
        let mut table = self;
        for segment in path.split('/') {
            if segment.is_empty() {
                continue;
            }

            if segment.starts_with('{') && segment.ends_with('}') {
                let name = &segment[1..segment.len() - 1];

                if table.wildcard.is_none() {
                    table.wildcard = Some(Box::new(Wildcard {
                        name: name.to_string(),
                        table: UrlTable::new(),
                    }));
                }

                match table.wildcard_mut().unwrap() {
                    Wildcard { name: n, .. } if name != n => {
                        panic!("Route {} segment `{{{}}}` conflicts with existing wildcard segment `{{{}}}`", path, name, n);
                    }
                    Wildcard { table: t, .. } => {
                        table = t;
                    }
                }
            } else {
                table = table
                    .next
                    .entry(segment.to_string())
                    .or_insert_with(UrlTable::new);
            }
        }

        table
    }
}

impl<R: Default> UrlTable<R> {
    /// Add or access a new resource at the given routing path (which may contain wildcards).
    #[allow(dead_code)]
    pub fn setup(&mut self, path: &str) -> &mut R {
        let table = self.setup_table(path);

        if table.accept.is_none() {
            table.accept = Some(R::default())
        }

        table.accept.as_mut().unwrap()
    }
}

pub struct Resources<'a, R> {
    stack: Vec<&'a UrlTable<R>>,
}

impl<'a, R> Iterator for Resources<'a, R> {
    type Item = &'a R;

    fn next(&mut self) -> Option<&'a R> {
        while let Some(table) = self.stack.pop() {
            self.stack.extend(table.next.values());
            if let Some(wildcard) = table.wildcard.as_ref() {
                self.stack.push(&wildcard.table);
            }
            if let Some(res) = &table.accept {
                return Some(res);
            }
        }
        None
    }
}

pub struct ResourcesMut<'a, R> {
    stack: Vec<&'a mut UrlTable<R>>,
}

impl<'a, R> Iterator for ResourcesMut<'a, R> {
    type Item = &'a mut R;

    fn next(&mut self) -> Option<&'a mut R> {
        while let Some(table) = self.stack.pop() {
            self.stack.extend(table.next.values_mut());
            if let Some(wildcard) = table.wildcard.as_mut() {
                self.stack.push(&mut wildcard.table);
            }
            if let Some(res) = &mut table.accept {
                return Some(res);
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_route_no_matches() {
        let table: UrlTable<()> = UrlTable::new();

        assert!(table.route("").is_none());
        assert!(table.route("/").is_none());
        assert!(table.route("//").is_none());
        assert!(table.route("foo").is_none());
        assert!(table.route("foo/bar").is_none());
    }

    #[test]
    fn root_matches() {
        let mut table: UrlTable<()> = UrlTable::new();
        table.setup("/");

        assert!(table.route("").is_some());
        assert!(table.route("/").is_some());
        assert!(table.route("//").is_some());

        assert!(table.route("foo").is_none());
        assert!(table.route("foo/bar").is_none());
    }

    #[test]
    fn one_fixed_segment_matches() {
        let mut table: UrlTable<()> = UrlTable::new();
        table.setup("foo");

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
        let mut table: UrlTable<()> = UrlTable::new();
        table.setup("foo");
        table.setup("bar");

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
        let mut table: UrlTable<()> = UrlTable::new();
        table.setup("foo/bar");

        assert!(table.route("").is_none());
        assert!(table.route("foo").is_none());

        assert!(table.route("foo/bar").is_some());
    }

    #[test]
    fn multiple_nested_fixed_segment_matches() {
        let mut table: UrlTable<()> = UrlTable::new();
        table.setup("foo/bar");
        table.setup("baz");
        table.setup("quux/twiddle/twibble");

        assert!(table.route("").is_none());
        assert!(table.route("foo").is_none());
        assert!(table.route("quux").is_none());

        assert!(table.route("foo/bar").is_some());
        assert!(table.route("baz").is_some());
        assert!(table.route("quux/twiddle/twibble").is_some());
    }

    #[test]
    fn overlap_nested_fixed_segment_matches() {
        let mut table: UrlTable<i32> = UrlTable::new();
        *table.setup("") = 0;
        *table.setup("foo") = 1;
        *table.setup("foo/bar") = 2;

        assert_eq!(*table.route("/").unwrap().0, 0);
        assert_eq!(*table.route("/foo").unwrap().0, 1);
        assert_eq!(*table.route("/foo/bar").unwrap().0, 2);

        assert_eq!(*table.route("").unwrap().0, 0);
        assert_eq!(*table.route("foo").unwrap().0, 1);
        assert_eq!(*table.route("foo/bar").unwrap().0, 2);
    }

    #[test]
    fn wildcard_matches() {
        let mut table: UrlTable<()> = UrlTable::new();
        table.setup("{}");

        assert!(table.route("").is_none());
        assert!(table.route("foo/bar").is_none());

        assert!(table.route("foo").is_some());
        assert!(table.route("bar").is_some());
    }

    #[test]
    fn nested_wildcard_matches() {
        let mut table: UrlTable<()> = UrlTable::new();
        table.setup("{}/{}");

        assert!(table.route("").is_none());
        assert!(table.route("foo").is_none());

        assert!(table.route("foo/bar").is_some());
        assert_eq!(&table.route("foo/bar").unwrap().1.vec, &["foo", "bar"]);
        assert!(table.route("foo/bar").unwrap().1.map.is_empty());
    }

    #[test]
    fn mixed_route() {
        let mut table: UrlTable<()> = UrlTable::new();
        table.setup("foo/{}/bar");

        assert!(table.route("").is_none());
        assert!(table.route("foo").is_none());
        assert!(table.route("foo/bar").is_none());
        assert!(table.route("foo/bar/baz").is_none());

        assert!(table.route("foo/baz/bar").is_some());
        assert_eq!(&table.route("foo/baz/bar").unwrap().1.vec, &["baz"]);
    }

    #[test]
    fn wildcard_fallback() {
        let mut table: UrlTable<i32> = UrlTable::new();
        *table.setup("foo") = 0;
        *table.setup("foo/bar") = 1;
        *table.setup("foo/{}/bar") = 2;

        assert!(table.route("").is_none());
        assert!(table.route("foo/bar/baz").is_none());
        assert!(table.route("foo/bar/bar").is_none());

        assert_eq!(*table.route("foo").unwrap().0, 0);
        assert_eq!(*table.route("foo/bar").unwrap().0, 1);
        assert_eq!(*table.route("foo/baz/bar").unwrap().0, 2);
    }
}
