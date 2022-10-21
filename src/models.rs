use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Action {
    CreateEpic,
    NavigateToEpicDetail{epic_id: u32},
    UpdateEpicStatus{epic_id: u32},
    DeleteEpic{epic_id: u32},
    CreateStory{epic_id: u32},
    NavigateToStoryDetail{epic_id: u32, story_id: u32},
    UpdateStoryStatus{story_id: u32},
    DeleteStory{epic_id: u32, story_id: u32},
    NavigateToPreviousPage,
    Exit
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum Status {
    Open,
    InProgress,
    Resolved,
    Closed
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "OPEN"),
            Self::InProgress => write!(f, "IN PROGRESS"),
            Self::Resolved => write!(f, "RESOLVED"),
            Self::Closed => write!(f, "CLOSED"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Epic {
    pub name: String,
    pub description: String,
    pub status: Status,
    pub stories: Vec<u32>,
}

impl Epic {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            status: Status::Open,
            stories: vec![]
        }
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Story {
    pub name: String,
    pub description: String,
    pub status: Status,
}

impl Story {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            status: Status::Open,
        }
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct DBState {
    pub last_item_id: u32,
    pub epics: HashMap<u32, Epic>,
    pub stories: HashMap<u32, Story>
}