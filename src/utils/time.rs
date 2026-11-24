use jiff::{Unit, ZonedDateTime};
use regex::Regex;

pub(crate) fn utc_now_timestamp_string() -> String {
    let now_time_string = format!("{}", ZonedDateTime::now().round(Unit::Second).unwrap());
    let now_time = now_time_string.as_str();
    let time_re = Regex::new(r"\..*").unwrap();
    let fixed_now_time = time_re.replace(now_time, ".000Z");
    let fixed_now_time_str = &*fixed_now_time;
    fixed_now_time_str.to_string()
}