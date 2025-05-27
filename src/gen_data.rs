use chrono::{Duration, NaiveDateTime, Utc};
use serde_json::json;
use std::fs::File;
use std::io::Write;

fn main() {
    let mut data = Vec::new();
    let base_time =
        NaiveDateTime::parse_from_str("2025-05-15 03:43:51", "%Y-%m-%d %H:%M:%S").unwrap();
    let budgets = vec![
        10, 20, 30, 40, 10, 20, 30, 40, 10, 20, 30, 40, 10, 20, 30, 40, 10, 20, 30, 40, 10, 20, 30,
        40,
    ];

    for i in 0..1000 {
        let signage_id = 7230 + i;
        let date = base_time + Duration::hours(i as i64);
        // ISO8601形式で出力
        let date_str = date.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        data.push(json!({
            "signage_id": signage_id,
            "date": { "$date": date_str },
            "budgets": budgets
        }));
    }

    let json_str = serde_json::to_string_pretty(&data).unwrap();
    let mut file = File::create("data_bson.json").unwrap();
    file.write_all(json_str.as_bytes()).unwrap();
}
