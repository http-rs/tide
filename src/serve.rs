// Use hyper to serve the given HttpService at the given address
pub(crate) fn serve<S: http_service::HttpService>(s: S, addr: std::net::SocketAddr) {
    http_service_hyper::serve(s, addr);
}
