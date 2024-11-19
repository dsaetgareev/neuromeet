
// This is read at compile time, please restart if you change this value.
// pub const LOGIN_URL: &str = std::env!("LOGIN_URL");
pub const LOGIN_URL: &str = "http://localhost:8080/login";
// pub const ACTIX_WEBSOCKET: &str = concat!(std::env!("ACTIX_UI_BACKEND_URL"), "/lobby");
pub const ACTIX_WEBSOCKET: &str =  "ws://localhost:8080/lobby";
// pub const WEBTRANSPORT_HOST: &str = concat!(std::env!("WEBTRANSPORT_HOST"), "/lobby");
pub const WEBTRANSPORT_HOST: &str =  "https://live-lesson.ru:4433/lobby";
// pub const WEBTRANSPORT_HOST: &str =  "https://127.0.0.1:4433/lobby";

pub const VIDEO_ELEMENT_ID: &str = "webcam";

pub fn split_users(s: Option<&str>) -> Vec<String> {
    if let Some(s) = s {
        s.split(',')
            .filter_map(|s| {
                let s = s.trim().to_string();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            })
            .collect::<Vec<String>>()
    } else {
        Vec::new()
    }
}

