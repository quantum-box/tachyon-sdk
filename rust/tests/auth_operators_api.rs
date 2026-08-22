use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

use tachyon_sdk::apis::{
    auth_operators_api::{delete_operator, DeleteOperatorError},
    configuration::Configuration,
    Error,
};

fn spawn_http_server(
    status: &'static str,
    response_body: &'static str,
) -> (String, mpsc::Receiver<String>) {
    let listener =
        TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let addr = listener.local_addr().expect("read local addr");
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept request");
        let mut buffer = [0_u8; 8192];
        let mut request = Vec::new();

        loop {
            let bytes_read =
                stream.read(&mut buffer).expect("read request");
            if bytes_read == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes_read]);

            if let Some(header_end) =
                request.windows(4).position(|window| window == b"\r\n\r\n")
            {
                let headers =
                    String::from_utf8_lossy(&request[..header_end]);
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        line.strip_prefix("content-length: ").or_else(
                            || line.strip_prefix("Content-Length: "),
                        )
                    })
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(0);

                if request.len() >= header_end + 4 + content_length {
                    break;
                }
            }
        }

        let captured = String::from_utf8(request).expect("request is utf8");
        tx.send(captured).expect("send captured request");

        let response = format!(
            "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{response_body}",
            response_body.len()
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
    });

    (format!("http://{addr}"), rx)
}

#[tokio::test]
async fn delete_operator_sends_delete_and_decodes_success() {
    let (base_path, captured_request) =
        spawn_http_server("200 OK", r#"{"success":true}"#);
    let configuration = Configuration {
        base_path,
        ..Configuration::default()
    };

    let response = delete_operator(&configuration, "tn_target123")
        .await
        .expect("delete operator succeeds");

    assert!(response.success);

    let request = captured_request.recv().expect("captured request");
    assert!(request
        .starts_with("DELETE /v1/auth/operators/tn_target123 HTTP/1.1"));
}

#[tokio::test]
async fn delete_operator_maps_not_found_response() {
    let (base_path, _captured_request) = spawn_http_server(
        "404 Not Found",
        r#"{"code":"NOT_FOUND","message":"Operator not found: tn_missing"}"#,
    );
    let configuration = Configuration {
        base_path,
        ..Configuration::default()
    };

    let error = delete_operator(&configuration, "tn_missing")
        .await
        .expect_err("delete operator returns typed error");

    match error {
        Error::ResponseError(content) => {
            assert_eq!(content.status.as_u16(), 404);
            match content.entity {
                Some(DeleteOperatorError::UnknownValue(value)) => {
                    assert_eq!(value["code"], "NOT_FOUND");
                }
                other => panic!("unexpected error entity: {other:?}"),
            }
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
