mod common;
use common::make_test_conf;

use pankosmia_web::rocket as my_rocket;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::local::blocking::Client;
use serde_json;
use serial_test::serial;

fn test_atomic(client: &Client, url: &str, method: &str, key: &str, value: bool) -> () {
    let response;
    if method == "post" {
        response = client.post(url).dispatch();
    } else {
        response = client.get(url).dispatch();
    }
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let response_string = response.into_string().expect("response as string");
    let response_json = serde_json::from_str::<serde_json::Value>(&response_string)
        .expect("response as json");
    assert_eq!(
        response_json[key].as_bool().expect("response as bool"),
        value
    );
}

#[test]
#[serial]
fn net() {
    let conf = make_test_conf();
    let client = Client::tracked(my_rocket(conf)).unwrap();
    test_atomic(&client, "/api/net/status", "get", "is_enabled", false);
    test_atomic(&client, "/api/net/enable", "post", "is_good", true);
    test_atomic(&client, "/api/net/status", "get", "is_enabled", true);
    test_atomic(&client, "/api/net/disable", "post", "is_good", true);
    test_atomic(&client, "/api/net/status", "get", "is_enabled", false);
}

#[test]
#[serial]
fn debug() {
    let conf = make_test_conf();
    let client = Client::tracked(my_rocket(conf)).unwrap();
    test_atomic(&client, "/api/debug/status", "get", "is_enabled", false);
    test_atomic(&client, "/api/debug/enable", "post", "is_good", true);
    test_atomic(&client, "/api/debug/status", "get", "is_enabled", true);
    test_atomic(&client, "/api/debug/disable", "post", "is_good", true);
    test_atomic(&client, "/api/debug/status", "get", "is_enabled", false);
}

#[test]
#[serial]
fn beta() {
    let conf = make_test_conf();
    let client = Client::tracked(my_rocket(conf)).unwrap();
    test_atomic(&client, "/api/beta/status", "get", "is_enabled", false);
    test_atomic(&client, "/api/beta/enable", "post", "is_good", true);
    test_atomic(&client, "/api/beta/status", "get", "is_enabled", true);
    test_atomic(&client, "/api/beta/disable", "post", "is_good", true);
    test_atomic(&client, "/api/beta/status", "get", "is_enabled", false);
}
