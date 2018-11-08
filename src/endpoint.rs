use futures::future::{Future, FutureObj};

use crate::{extract::Extract, head::Head, IntoResponse, Request, Response, RouteMatch};

/// The raw representation of an endpoint.
///
/// This trait is automatically implemented by a host of `Fn` types, and should not be
/// implemented directly by Tide users.
pub trait Endpoint<Data, Kind>: Send + Sync + 'static {
    /// The async result of `call`.
    type Fut: Future<Output = (Head, Response)> + Send + 'static;

    /// Invoke the endpoint on the given request and app data handle.
    fn call(&self, data: Data, req: Request, params: RouteMatch<'_>) -> Self::Fut;
}

pub(crate) struct BoxedEndpoint<Data> {
    endpoint: Box<
        dyn Fn(Data, Request, RouteMatch) -> FutureObj<'static, (Head, Response)> + Send + Sync,
    >,
}

impl<Data> BoxedEndpoint<Data> {
    pub fn new<T, Kind>(ep: T) -> BoxedEndpoint<Data>
    where
        T: Endpoint<Data, Kind>,
    {
        BoxedEndpoint {
            endpoint: Box::new(move |data, request, params| {
                FutureObj::new(Box::new(ep.call(data, request, params)))
            }),
        }
    }

    pub fn call(
        &self,
        data: Data,
        req: Request,
        params: RouteMatch<'_>,
    ) -> FutureObj<'static, (Head, Response)> {
        (self.endpoint)(data, req, params)
    }
}

/// A marker type used for the (phantom) `Kind` parameter in endpoints.
#[doc(hidden)]
pub struct Ty<T>(T);

macro_rules! end_point_impl_with_head {
    ($($X:ident),*) => {
        impl<T, Data, Fut, $($X),*> Endpoint<Data, (Ty<Fut>, Head, $(Ty<$X>),*)> for T
        where
            T: Send + Sync + Clone + 'static + Fn(Head, $($X),*) -> Fut,
            Data: Clone + Send + Sync + 'static,
            Fut: Future + Send + 'static,
            Fut::Output: IntoResponse,
            $(
                $X: Extract<Data>
            ),*
        {
            type Fut = FutureObj<'static, (Head, Response)>;

            #[allow(unused_mut, non_snake_case)]
            fn call(&self, mut data: Data, mut req: Request, params: RouteMatch<'_>) -> Self::Fut {
                let f = self.clone();
                $(let $X = $X::extract(&mut data, &mut req, &params);)*
                FutureObj::new(Box::new(async move {
                    let (parts, _) = req.into_parts();
                    let head = Head::from(parts);
                    $(let $X = match await!($X) {
                        Ok(x) => x,
                        Err(resp) => return (head, resp),
                    };)*
                    let res = await!(f(head.clone(), $($X),*));

                    (head, res.into_response())
                }))
            }
        }
    };
}

// TODO: refactor to share code with the above macro
macro_rules! end_point_impl_no_head {
    ($($X:ident),*) => {
        impl<T, Data, Fut, $($X),*> Endpoint<Data, (Ty<Fut>, $(Ty<$X>),*)> for T
        where
            T: Send + Sync + Clone + 'static + Fn($($X),*) -> Fut,
            Data: Clone + Send + Sync + 'static,
            Fut: Future + Send + 'static,
            Fut::Output: IntoResponse,
            $(
                $X: Extract<Data>
            ),*
        {
            type Fut = FutureObj<'static, (Head, Response)>;

            #[allow(unused_mut, non_snake_case)]
            fn call(&self, mut data: Data, mut req: Request, params: RouteMatch<'_>) -> Self::Fut {
                let f = self.clone();
                $(let $X = $X::extract(&mut data, &mut req, &params);)*
                FutureObj::new(Box::new(async move {
                    let (parts, _) = req.into_parts();
                    let head = Head::from(parts);
                    $(let $X = match await!($X) {
                        Ok(x) => x,
                        Err(resp) => return (head, resp),
                    };)*
                    let res = await!(f($($X),*));

                    (head, res.into_response())
                }))
            }
        }
    };
}

macro_rules! end_point_impl {
    ($($X:ident),*) => {
        end_point_impl_with_head!($($X),*);
        end_point_impl_no_head!($($X),*);
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
