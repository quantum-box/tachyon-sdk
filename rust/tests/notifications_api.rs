use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

use tachyon_sdk::apis::{
    configuration::Configuration,
    notifications_api::{send_sms_notification, SendSmsNotificationError},
    Error,
};
use tachyon_sdk::models::SendSmsNotificationRequest;

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

fn sms_request() -> SendSmsNotificationRequest {
    SendSmsNotificationRequest::new(
        "+15551234567".to_string(),
        "Your verification code is 123456.".to_string(),
    )
}

#[tokio::test]
async fn send_sms_notification_posts_expected_payload() {
    let (base_path, captured_request) =
        spawn_http_server("202 Accepted", r#"{"accepted":true}"#);
    let configuration = Configuration {
        base_path,
        ..Configuration::default()
    };

    let response = send_sms_notification(
        &configuration,
        "tn_test",
        "Bearer test-token",
        sms_request(),
    )
    .await
    .expect("send sms notification succeeds");

    assert!(response.accepted);

    let request = captured_request.recv().expect("captured request");
    assert!(request.starts_with("POST /v1/notifications/sms HTTP/1.1"));
    assert!(request.contains("x-operator-id: tn_test"));
    assert!(request.contains("authorization: Bearer test-token"));
    assert!(request.contains("content-type: application/json"));
    assert!(request.ends_with(
        r#"{"phoneNumber":"+15551234567","message":"Your verification code is 123456."}"#
    ));
}

#[tokio::test]
async fn send_sms_notification_maps_error_response() {
    let (base_path, _captured_request) = spawn_http_server(
        "400 Bad Request",
        r#"{"message":"invalid phone number"}"#,
    );
    let configuration = Configuration {
        base_path,
        ..Configuration::default()
    };

    let error = send_sms_notification(
        &configuration,
        "tn_test",
        "Bearer test-token",
        sms_request(),
    )
    .await
    .expect_err("send sms notification returns typed error");

    match error {
        Error::ResponseError(content) => {
            assert_eq!(content.status.as_u16(), 400);
            match content.entity {
                Some(SendSmsNotificationError::Status400(
                    error_response,
                )) => {
                    assert_eq!(
                        error_response.message,
                        "invalid phone number"
                    );
                }
                other => panic!("unexpected error entity: {other:?}"),
            }
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
