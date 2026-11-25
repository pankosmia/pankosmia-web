mod test {
    use pankosmia_web::rocket as my_rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use serde_json::{Value};
    use rocket::http::ContentType;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::absolute;

    pub(crate) fn make_test_conf() -> Value {
        let conf_path = absolute("./test_data/test_config.json").unwrap();
        let file = match File::open(&conf_path) {
            Ok(f) => f,
            Err(e) => {panic!("Could not open conf file {:?}: {}", &conf_path, e);}
        };
        let reader = BufReader::new(file);
        let conf_json = serde_json::from_reader(reader).unwrap();
        conf_json
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
