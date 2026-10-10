use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct BubbleAction {
    pub id: String,
    pub label: String,
}

pub fn action(id: &str, label: &str) -> BubbleAction {
    BubbleAction {
        id: id.into(),
        label: label.into(),
    }
}
