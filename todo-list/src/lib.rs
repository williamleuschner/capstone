use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct IncomingTodo {
    pub title: String,
    pub startable: NaiveDate,
    pub due: NaiveDate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub complete: bool,
    pub startable: NaiveDate,
    pub due: NaiveDate,
}

pub struct TodoList {
    items: HashMap<Uuid, Todo>,
}

impl Default for TodoList {
    fn default() -> Self {
        Self::new()
    }
}

impl TodoList {
    pub fn new() -> TodoList {
        TodoList {
            items: HashMap::new(),
        }
    }
    pub fn add(&mut self, new: IncomingTodo) -> Todo {
        let created = Todo {
            id: Uuid::new_v4(),
            title: new.title,
            complete: false,
            startable: new.startable,
            due: new.due,
        };
        let result = created.clone();
        self.items.insert(created.id, created);
        result
    }

    pub fn toggle_completed(&mut self, id: Uuid) {
        let maybe_item = self.items.get(&id);
        println!("found item: {:?}", maybe_item);
        if let Some(item) = maybe_item {
            let toggled_item = Todo {
                id: item.id,
                title: item.title.clone(),
                complete: !item.complete,
                startable: item.startable,
                due: item.due,
            };
            println!("updated item: {:?}", toggled_item);
            let old_item = self.items.insert(toggled_item.id, toggled_item);
            println!("old item from hashmap: {:?}", old_item);
            let gotten_item = self.items.get(&id);
            println!("what you get from the hashmap: {:?}", gotten_item);
        };
    }

    pub fn update(&mut self, existing: Todo) {
        self.items.insert(existing.id, existing);
    }

    pub fn get_all(&mut self) -> Vec<Todo> {
        let mut result = Vec::with_capacity(self.items.len());
        for todo in self.items.values() {
            result.push(todo.clone());
        }
        result
    }

    pub fn get(&mut self, id: Uuid) -> Option<Todo> {
        self.items.get(&id).cloned()
    }
}
