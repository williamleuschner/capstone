use actix_web::{get, post, web, App, HttpServer, HttpResponse, Responder};
use chrono::Utc;
use todo_list::{IncomingTodo, Todo, TodoList};

#[get("/greeting/{name}")]
async fn index(web::Path(name): web::Path<String>) -> impl Responder {
    format!("Hello {}!", name)
}

#[post("/todo/create")]
async fn todo(body: web::Bytes) -> impl Responder {
    let title = std::str::from_utf8(&body).unwrap().to_owned();
    let date = Utc::now();
    let new = IncomingTodo {
        title: title,
        startable: date,
        due: date,
    };
    let mut todos = TodoList::new();
    todos.add(new);
    let all_todos = todos.get_all();
    let new_json = serde_json::to_string(&all_todos);

    match new_json {
        Ok(result) => HttpResponse::Ok()
            .content_type("application/json")
            .body(result),
        Err(_) => HttpResponse::InternalServerError().body("Whoops".to_owned())
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // let mut todos = TodoList::new();
    HttpServer::new(|| App::new().service(index).service(todo))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
