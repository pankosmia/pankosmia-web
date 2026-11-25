use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::path::absolute;

pub(crate) fn make_test_conf() -> Value {
    let conf_path = absolute("./test_data/test_config.json").unwrap();
    let file = match File::open(&conf_path) {
        Ok(f) => f,
        Err(e) => {
            panic!("Could not open conf file {:?}: {}", &conf_path, e);
        }
    };
    let reader = BufReader::new(file);
    let conf_json = serde_json::from_reader(reader).unwrap();
    conf_json
}
