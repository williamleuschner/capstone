use tera::{Context, Tera};
use hyper::Body;
use thruster::context::hyper_request::HyperRequest;
use thruster::context::typed_hyper_context::TypedHyperContext;
use thruster::hyper_server::HyperServer;
use thruster::{async_middleware, middleware_fn};
use thruster::{App, ThrusterServer};
use thruster::{MiddlewareNext, MiddlewareResult};
use chrono::Utc;
use todo_list::{IncomingTodo, Todo, TodoList};

use std::sync::{Arc, RwLock};

type Ctx = TypedHyperContext<RequestConfig>;

struct ServerConfig {
    tera: Arc<RwLock<tera::Tera>>,
    todos: Arc<RwLock<TodoList>>,
}

struct RequestConfig {
    tera: Arc<RwLock<tera::Tera>>,
    todos: Arc<RwLock<TodoList>>,
}

fn generate_context(request: HyperRequest, state: &ServerConfig, _path: &str) -> Ctx {
    Ctx::new(
        request,
        RequestConfig {
            tera: state.tera.clone(),
            todos: state.todos.clone(),
        },
    )
}

fn not_found_404(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.body = Body::from("<!DOCTYPE html>
    <html lang=\"en\">
    <head>
    <meta charset=\"utf-8\">
    <title>404 Not Found</title>
    </head>
    <body>
    <h1>404 Not Found</h1>
    </body>
    </html>");
    context.content_type("text/html");
    context.status(404);

    Ok(context)
}

// fn greet(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
//     // let mut context = Ctx::new(context);
//
//     println!("{:?}", context.headers);
//     if let Some(name) = context.params.get("name") {
//         context.body = Body::from(format!("Hello, {}!", name));
//     } else {
//         context.body = Body::from("Hello, World!");
//     }
//
//     context.set_header("Server".to_owned(), "thruster".to_owned());
//     context.set_header("Content-Type".to_owned(), "text/plain".to_owned());
//
//     Box::new(future::ok(context))
// }

// #[middleware_fn]
// async fn post_new_todo(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
//     let todos = context.extra.todos.clone();
//     let mut todos = todos.write().unwrap();
//
//     let success = if let Some(title) = query.get("title") {
//         if let Some(due_date) = query.get("due-date") {
//             if let Some(start_date) = query.get("start-date") {
//                 let new = IncomingTodo {
//                     title: title.to_string(),
//                     startable: NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
//                         .unwrap_or_else(|_| NaiveDate::from_ymd(1970, 1, 1)),
//                     due: NaiveDate::parse_from_str(due_date, "%Y-%m-%d")
//                         .unwrap_or_else(|_| NaiveDate::from_ymd(1970, 1, 1)),
//                 };
//                 let mut mutexed_todo_state = todo_state.list.lock().unwrap();
//                 let mutexed_todos = mutexed_todo_state.deref_mut();
//                 mutexed_todos.add(new);
//                 true
//             } else {
//                 false
//             }
//         } else {
//             false
//         }
//     } else {
//         false
//     };
//     todos.
//     // *latest_value = context
//     //     .params
//     //     .as_ref()
//     //     .unwrap()
//     //     .get("val")
//     //     .unwrap()
//     //     .to_string();
//
//     context.redirect("/");
//
//     Ok(context)
// }

fn post_edit_todo() {}

#[middleware_fn]
async fn get_new_todo(mut request_context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let tera = request_context.extra.tera.clone();
    let tera = tera.read().unwrap();
    let mut tpl_context = Context::new();
    tpl_context.insert("action", "Create");
    if let Ok(s) = tera.render("detail.html.j2", &tpl_context) {
        request_context.body = Body::from(s);
    } else {
        request_context.body = Body::from("oops");
    }

    Ok(request_context)
}

fn get_edit_todo() {}

fn post_complete_todo() {}

#[middleware_fn]
async fn get_index(mut req_context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let tera = req_context.extra.tera.clone();
    let tera = tera.read().unwrap();
    let todos = req_context.extra.todos.clone();
    let mut todos = todos.write().unwrap();
    let mut tpl_context = Context::new();

    let all_todos = todos.get_all();
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
    tpl_context.insert("today_todos", &today_todos);
    tpl_context.insert("upcoming_todos", &upcoming_todos);

    if let Ok(s) = tera.render("index.html.j2", &tpl_context) {
        req_context.body = Body::from(s);
    } else {
        req_context.body = Body::from("template error");
    }

    Ok(req_context)
}

fn main() {
    println!("Starting server...");

    let mut tera = match Tera::new("templates/*.j2") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    tera.autoescape_on(vec!["html.j2"]);

    let mut app = App::<HyperRequest, Ctx, ServerConfig>::create(
        generate_context,
        ServerConfig {
            tera: Arc::new(RwLock::new(tera)),
            todos: Arc::new(RwLock::new(TodoList::new())),
        },
    );

    // _app.use_middleware("/", profiling);

    // app.get("/greeting/:name", vec![greet]);
    // app.post("/new", async_middleware!(Ctx, [post_new_todo]));
    app.get("/", async_middleware!(Ctx, [get_index]));
    app.get("/new", async_middleware!(Ctx, [get_new_todo]));
    // app.set404(vec![not_found_404]);

    let server = HyperServer::new(app);
    server.start("0.0.0.0", 8080);
}
