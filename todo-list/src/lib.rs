use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct IncomingTodo {
    pub title: String,
    pub startable: DateTime<Utc>,
    pub due: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub complete: bool,
    pub startable: DateTime<Utc>,
    pub due: DateTime<Utc>,
}

pub struct TodoList {
    items: HashMap<Uuid, Todo>
}

impl TodoList {
    pub fn new() -> TodoList {
        TodoList {
            items: HashMap::new(),
        }
    }
    pub fn add(&mut self, new: IncomingTodo) {
        let created = Todo {
            id: Uuid::new_v4(),
            title: new.title,
            complete: false,
            startable: new.startable,
            due: new.due,
        };
        self.items.insert(created.id, created);
    }

    pub fn mark_completed(&mut self, id: Uuid) {
        let maybe_item = self.items.get(&id);
        if let Some(item) = maybe_item {
            let completed_item = Todo {
                id: item.id,
                title: item.title.clone(),
                complete: true,
                startable: item.startable,
                due: item.due,
            };
            self.items.insert(completed_item.id, completed_item);
        };
    }

    pub fn get_all(&mut self) -> Vec<Todo> {
        Vec::new()
    }
}
