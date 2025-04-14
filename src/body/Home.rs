use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct HomeCreate<'a> {
    pub name: &'a str,
}