// untils/question.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub title: String,
    pub description: String,
    pub templates: Option<Templates>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Templates {
    pub cpp: Option<String>,
    pub python: Option<String>,
    pub rust: Option<String>,
}

impl Question {
    pub fn new(id: String, title: String, description: String) -> Self {
        Question {
            id,
            title,
            description,
            templates: None,
        }
    }

    pub fn with_templates(mut self, templates: Templates) -> Self {
        self.templates = Some(templates);
        self
    }
}