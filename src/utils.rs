use redis::{SetExpiry, SetOptions};

pub fn get_redis_id(route: &str, id: &String) -> String {
    format!("{}_{}", route, id)
}

pub fn get_redis_set_options() -> SetOptions {
    SetOptions::default().with_expiration(SetExpiry::EX(30))
}
