pub async fn determine_port_to_bind() -> async_std::net::SocketAddr {
    async_std::net::TcpListener::bind("localhost:0")
        .await
        .unwrap()
        .local_addr()
        .unwrap()
}
