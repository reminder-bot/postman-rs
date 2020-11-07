use serde_json::json as s_json;

#[macro_export]
macro_rules! json {
    ($($json:tt)+) => {
        s_json!($($json)+).as_object().unwrap()
    };
}
