use serde::*;

#[derive(Default, Debug, Clone, Deserialize)]
pub struct PageOptions {
    pub title: String,
}
