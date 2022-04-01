use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Note {
    pub method: String,
    pub id: i64,
    pub created: i64,
    pub changed: i64,
    pub title: String,
    pub content: String,
}

pub type NoteVector = Vec<Note>;