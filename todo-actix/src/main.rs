use actix_web::{error, get, post, web, App, Error, HttpResponse, HttpServer, Responder};
use chrono::{NaiveDate, Utc};
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Mutex;
use tera::{Context, Tera};
use todo_list::{IncomingTodo, Todo, TodoList};
use uuid::Uuid;

struct AppStateWithTodoList {
    list: Mutex<TodoList>,
}

fn redirect(to: &str) -> HttpResponse {
    HttpResponse::Found().header("Location", to).finish()
}

#[get("/greeting/{name}")]
async fn greet(web::Path(name): web::Path<String>) -> impl Responder {
    format!("Hello {}!", name)
}

#[post("/new")]
async fn post_new_todo(
    todo_state: web::Data<AppStateWithTodoList>,
    query: web::Form<HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
    let success = if let Some(title) = query.get("title") {
        if let Some(due_date) = query.get("due-date") {
            if let Some(start_date) = query.get("start-date") {
                let new = IncomingTodo {
                    title: title.to_string(),
                    startable: NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
                        .unwrap_or_else(|_| NaiveDate::from_ymd(1970, 1, 1)),
                    due: NaiveDate::parse_from_str(due_date, "%Y-%m-%d")
                        .unwrap_or_else(|_| NaiveDate::from_ymd(1970, 1, 1)),
                };
                let mut mutexed_todo_state = todo_state.list.lock().unwrap();
                let mutexed_todos = mutexed_todo_state.deref_mut();
                mutexed_todos.add(new);
                true
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    if success {
        Ok(redirect("/"))
    } else {
        Err(error::ErrorInternalServerError("Whoops"))
    }
}

#[post("/edit/{id}")]
async fn post_edit_todo(
    web::Path(id): web::Path<String>,
    todo_state: web::Data<AppStateWithTodoList>,
    query: web::Form<HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
    println!("POST /edit/{}", id);
    let success = if let Some(title) = query.get("title") {
        if let Some(due_date) = query.get("due-date") {
            if let Some(start_date) = query.get("start-date") {
                if let Ok(uuid) = Uuid::parse_str(&id) {
                    let mut mutexed_todo_state = todo_state.list.lock().unwrap();
                    let mutexed_todos = mutexed_todo_state.deref_mut();
                    if let Some(existing) = mutexed_todos.get(uuid) {
                        let updated = Todo {
                            id: existing.id,
                            title: title.to_string(),
                            complete: existing.complete,
                            startable: NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
                                .unwrap_or_else(|_| NaiveDate::from_ymd(1970, 1, 1)),
                            due: NaiveDate::parse_from_str(due_date, "%Y-%m-%d")
                                .unwrap_or_else(|_| NaiveDate::from_ymd(1970, 1, 1)),
                        };
                        mutexed_todos.update(updated);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    if success {
        Ok(redirect("/"))
    } else {
        Err(error::ErrorInternalServerError("Whoops"))
    }
}

#[get("/new")]
async fn get_new_todo(tera: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let mut context = Context::new();
    context.insert("action", "Create");
    let s = tera
        .render("detail.html.j2", &context)
        .map_err(|e| error::ErrorInternalServerError(format!("Template error: {:?}", e)))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[get("/edit/{id}")]
async fn get_edit_todo(
    web::Path(id): web::Path<String>,
    todo_state: web::Data<AppStateWithTodoList>,
    tera: web::Data<tera::Tera>,
) -> Result<HttpResponse, Error> {
    println!("GET /edit/{}", id);
    if let Ok(uuid) = Uuid::parse_str(&id) {
        let mut mutexed_todos_state = todo_state.list.lock().unwrap();
        let mutexed_todos = mutexed_todos_state.deref_mut();
        if let Some(this_todo) = mutexed_todos.get(uuid) {
            let mut context = Context::new();
            context.insert("todo", &this_todo);
            context.insert("action", "Update");
            let s = tera
                .render("detail.html.j2", &context)
                .map_err(|e| error::ErrorInternalServerError(format!("Template error: {:?}", e)))?;
            Ok(HttpResponse::Ok().content_type("text/html").body(s))
        } else {
            Err(error::ErrorNotFound("unknown uuid"))
        }
    } else {
        Err(error::ErrorNotFound("bad uuid"))
    }
}

#[post("/complete/{id}")]
async fn post_complete_todo(
    web::Path(id): web::Path<String>,
    todo_state: web::Data<AppStateWithTodoList>,
) -> Result<HttpResponse, Error> {
    println!("POST /complete/{}", id);
    if let Ok(uuid) = Uuid::parse_str(&id) {
        let mut mutexed_todos_state = todo_state.list.lock().unwrap();
        let mutexed_todos = mutexed_todos_state.deref_mut();
        mutexed_todos.toggle_completed(uuid);
        let s = "{}";
        Ok(HttpResponse::Ok().content_type("text/html").body(s))
    } else {
        Err(error::ErrorNotFound("unknown uuid"))
    }
}

#[get("/")]
async fn get_index(
    todo_state: web::Data<AppStateWithTodoList>,
    tera: web::Data<tera::Tera>,
) -> Result<HttpResponse, Error> {
    let mut context = Context::new();
    let mut mutexed_todos = todo_state.list.lock().unwrap();
    let all_todos = &mutexed_todos.deref_mut().get_all();
    let mut today_todos = Vec::new();
    let mut upcoming_todos = Vec::new();
    let today = Utc::today().naive_utc();
    for a_todo in all_todos {
        if a_todo.startable <= today {
            today_todos.push(a_todo);
        } else {
            upcoming_todos.push(a_todo);
        }
    }
    context.insert("today_todos", &today_todos);
    context.insert("upcoming_todos", &upcoming_todos);
    let s = tera
        .render("index.html.j2", &context)
        .map_err(|e| error::ErrorInternalServerError(format!("Template error: {:?}", e)))?;
    // let s = "hello world".to_owned();
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let todo_state = web::Data::new(AppStateWithTodoList {
        list: Mutex::new(TodoList::new()),
    });

    HttpServer::new(move || {
        let mut tera = match Tera::new("templates/*.j2") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec!["html.j2"]);

        App::new()
            // .app_data(todos.clone())
            .app_data(todo_state.clone())
            .data(tera)
            .service(greet)
            .service(get_index)
            .service(post_new_todo)
            .service(get_new_todo)
            .service(get_edit_todo)
            .service(post_edit_todo)
            .service(post_complete_todo)
            .service(actix_files::Files::new("/static", "./static"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
