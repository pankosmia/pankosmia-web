mod test {
    use pankosmia_web::rocket as my_rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use rocket::fs::relative;
    use serde_json::{Value, json};
    use rocket::http::ContentType;

    pub(crate) fn make_test_conf() -> Value {
        let app_resources_parent_path = relative!("../linux/build/lib/").to_string();
        let app_resources_path = format!("{}/app_resources/", &app_resources_parent_path);
        let working_dir = "/home/mark/pankosmia/pithekos".to_string();
        let webfont_path = format!("{}webfonts", app_resources_path);
        let app_setup_path = format!("{}setup/app_setup.json", app_resources_parent_path);
        let local_setup_path = relative!("local_test_setup.json");
        json!({
            "working_dir": working_dir,
            "webfont_path": webfont_path,
            "app_setup_path": app_setup_path,
            "local_setup_path": local_setup_path,
            "app_resources_path": app_resources_path,
        })
    }

    #[test]
    fn version() {
        let conf = make_test_conf();
        let client = Client::tracked(my_rocket(conf)).unwrap();
        let response = client.get("/api/version").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        let response_string = response.into_string().expect("version response as string");
        let response_json = serde_json::from_str::<serde_json::Value>(&response_string)
            .expect("version response as json");
        assert_eq!(
            response_json["os"].as_str().expect("response os as str"),
            "linux"
        );
    }
}
