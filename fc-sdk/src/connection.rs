use std::path::Path;

use firecracker_api::Client;

/// Creates a `firecracker_api::Client` connected via Unix socket.
pub fn connect(socket_path: impl AsRef<Path>) -> Client {
    let socket_path = socket_path.as_ref();
    let client = reqwest::Client::builder()
        .unix_socket(socket_path)
        .build()
        .expect("failed to build reqwest client with unix socket");
    // The base URL host is ignored for Unix sockets; we use "http://localhost".
    Client::new_with_client("http://localhost", client)
}
