use chrono::prelude::*;

/// get current_time
pub fn get_current_time() -> String {
    let dt: DateTime<Local> = Local::now();
    dt.format("%Y-%m-%d %H:%M:%S.%f").to_string()
}
/// get current unix time
pub fn get_unix_time() -> u64 {
    Local::now().timestamp_millis() as u64
}

/// covert date to unix time
pub fn time2unix(time_str: String) -> u64 {
    let dt = Utc
        .datetime_from_str(time_str.as_str(), "%Y-%m-%d %H:%M:%S.%f")
        .unwrap();
    dt.timestamp_millis() as u64
}
