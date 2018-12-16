use futures::future::{Future, FutureObj};

use crate::{
    configuration::Store, extract::Extract, head::Head, IntoResponse, Request, Response, RouteMatch,
};

/// The raw representation of an endpoint.
///
/// This trait is automatically implemented by a host of `Fn` types, and should not be
/// implemented directly by Tide users.
///
/// # Examples
///
/// Endpoints are implemented as asynchronous functions that make use of language features
/// currently only available in Rust Nightly. For this reason, we have to explicitly enable
/// those features with `#![feature(async_await, futures_api)]`. To keep examples concise,
/// the attribute will be omitted in most of the documentation.
///
/// A simple endpoint that is invoked on a `GET` request and returns a `String`:
///
/// ```rust, no_run
/// # #![feature(async_await, futures_api)]
/// async fn hello() -> String {
///     String::from("hello")
/// }
///
/// fn main() {
///     let mut app = tide::App::new(());
///     app.at("/hello").get(hello);
///     app.serve("127.0.0.1:7878")
/// }
/// ```
///
/// Endpoint accessing `App` state (`Data`) and body of `POST` request as `String`:
///
/// ```rust, no_run
/// # #![feature(async_await, futures_api)]
/// use std::sync::Arc;
/// use std::sync::Mutex;
/// use tide::AppData;
/// use tide::body;
///
/// #[derive(Clone)]
/// struct Database {
///     contents: Arc<Mutex<Vec<String>>>,
/// }
///
/// impl Database {
///     fn new() -> Database {
///         Database {
///             contents: Arc::new(Mutex::new(Vec::new())),
///         }
///     }
/// }
///
/// async fn insert(
///     mut db: AppData<Database>,
///     msg: body::Str,
/// ) -> String {
///     // insert into db
///     # String::from("")
/// }
///
/// fn main() {
///     let mut app = tide::App::new(Database::new());
///     app.at("/messages/insert").post(insert);
///     app.serve("127.0.0.1:7878")
/// }
/// ```
///
/// See [`body`](body/index.html) module for examples of how to work with request and response bodies.
///
pub trait Endpoint<Data, Kind>: Send + Sync + 'static {
    /// The async result of `call`.
    type Fut: Future<Output = Response> + Send + 'static;

    /// Invoke the endpoint on the given request and app data handle.
    fn call(
        &self,
        data: Data,
        req: Request,
        params: Option<RouteMatch<'_>>,
        store: &Store,
    ) -> Self::Fut;
}

type BoxedEndpointFn<Data> =
    dyn Fn(Data, Request, Option<RouteMatch>, &Store) -> FutureObj<'static, Response> + Send + Sync;

pub(crate) struct BoxedEndpoint<Data> {
    endpoint: Box<BoxedEndpointFn<Data>>,
}

impl<Data> BoxedEndpoint<Data> {
    pub fn new<T, Kind>(ep: T) -> BoxedEndpoint<Data>
    where
        T: Endpoint<Data, Kind>,
    {
        BoxedEndpoint {
            endpoint: Box::new(move |data, request, params, store| {
                FutureObj::new(Box::new(ep.call(data, request, params, store)))
            }),
        }
    }

    pub fn call(
        &self,
        data: Data,
        req: Request,
        params: Option<RouteMatch<'_>>,
        store: &Store,
    ) -> FutureObj<'static, Response> {
        (self.endpoint)(data, req, params, store)
    }
}

/// A marker type used for the (phantom) `Kind` parameter in endpoints.
#[doc(hidden)]
pub struct Ty<T>(T);

macro_rules! call_f {
    ($head_ty:ty; ($f:ident, $head:ident); $($X:ident),*) => {
        $f($head.clone(), $($X),*)
    };
    (($f:ident, $head:ident); $($X:ident),*) => {
        $f($($X),*)
    };
}

macro_rules! end_point_impl_raw {
    ($([$head:ty])* $($X:ident),*) => {
        impl<T, Data, Fut, $($X),*> Endpoint<Data, (Ty<Fut>, $($head,)* $(Ty<$X>),*)> for T
        where
            T: Send + Sync + Clone + 'static + Fn($($head,)* $($X),*) -> Fut,
            Data: Clone + Send + Sync + 'static,
            Fut: Future + Send + 'static,
            Fut::Output: IntoResponse,
            $(
                $X: Extract<Data>
            ),*
        {
            type Fut = FutureObj<'static, Response>;

            #[allow(unused_mut, non_snake_case)]
            fn call(&self, mut data: Data, mut req: Request, params: Option<RouteMatch<'_>>, store: &Store) -> Self::Fut {
                let f = self.clone();
                $(let $X = $X::extract(&mut data, &mut req, &params, store);)*
                FutureObj::new(Box::new(async move {
                    let (parts, _) = req.into_parts();
                    let head = Head::from(parts);
                    $(let $X = match await!($X) {
                        Ok(x) => x,
                        Err(resp) => return resp,
                    };)*
                    let res = await!(call_f!($($head;)* (f, head); $($X),*));

                    res.into_response()
                }))
            }
        }
    };
}

macro_rules! end_point_impl {
    ($($X:ident),*) => {
        end_point_impl_raw!([Head] $($X),*);
        end_point_impl_raw!($($X),*);
    }
}

end_point_impl!();
end_point_impl!(T0);
end_point_impl!(T0, T1);
end_point_impl!(T0, T1, T2);
end_point_impl!(T0, T1, T2, T3);
end_point_impl!(T0, T1, T2, T3, T4);
end_point_impl!(T0, T1, T2, T3, T4, T5);
end_point_impl!(T0, T1, T2, T3, T4, T5, T6);
end_point_impl!(T0, T1, T2, T3, T4, T5, T6, T7);
end_point_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
end_point_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
