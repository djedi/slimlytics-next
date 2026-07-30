use slimlytics_cli::ApiClient;
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};

#[tokio::test]
async fn authenticated_requests_do_not_follow_redirects() {
    let destination = TcpListener::bind("127.0.0.1:0").unwrap();
    let destination_address = destination.local_addr().unwrap();
    let destination_server = thread::spawn(move || {
        destination.set_nonblocking(true).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            match destination.accept() {
                Ok((mut stream, _)) => {
                    let mut request = [0_u8; 2048];
                    let count = stream.read(&mut request).unwrap();
                    let request = String::from_utf8_lossy(&request[..count]);
                    let leaked_authorization = request
                        .to_ascii_lowercase()
                        .contains("authorization: bearer secret-token");
                    stream
                        .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n[]")
                        .unwrap();
                    return Some(leaked_authorization);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if std::time::Instant::now() >= deadline {
                        return None;
                    }
                    thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(error) => panic!("destination accept failed: {error}"),
            }
        }
    });

    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let origin_address = origin.local_addr().unwrap();
    let origin_server = thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        let mut request = [0_u8; 2048];
        let _ = stream.read(&mut request).unwrap();
        let response = format!(
            "HTTP/1.1 307 Temporary Redirect\r\nLocation: http://{destination_address}/stolen\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        );
        stream.write_all(response.as_bytes()).unwrap();
    });

    let client = ApiClient::new(
        &format!("http://{origin_address}"),
        Some("secret-token".into()),
    )
    .unwrap();
    let result = client.sites().await;

    origin_server.join().unwrap();
    let redirected_request = destination_server.join().unwrap();
    assert!(result.is_err(), "redirect response must be rejected");
    assert!(
        redirected_request.is_none(),
        "redirect target must never receive the request"
    );
}
