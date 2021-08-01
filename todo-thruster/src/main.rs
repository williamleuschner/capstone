use chrono::{NaiveDate, Utc};
use hyper::Body;
use tera::{Context, Tera};
use thruster::context::hyper_request::HyperRequest;
use thruster::context::typed_hyper_context::TypedHyperContext;
use thruster::hyper_server::HyperServer;
use thruster::middleware::query_params::query_params;
use thruster::{async_middleware, middleware_fn};
use thruster::{App, ThrusterServer};
use thruster::{MiddlewareNext, MiddlewareResult};
use thruster::middleware::file::file;
use todo_list::{IncomingTodo, Todo, TodoList};
use uuid::Uuid;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thruster::errors::ThrusterError;

type Ctx = TypedHyperContext<RequestConfig>;

struct ServerConfig {
    tera: Arc<RwLock<tera::Tera>>,
    todos: Arc<RwLock<TodoList>>,
}

struct RequestConfig {
    tera: Arc<RwLock<tera::Tera>>,
    todos: Arc<RwLock<TodoList>>,
}

// I grabbed this function from the revision history of src/context/basic_hyper_context.rs because the author removed it as part of a "bug fix" that broke one of the examples.
#[middleware_fn]
async fn to_owned_request(
    context: Ctx,
    next: MiddlewareNext<Ctx>,
) -> MiddlewareResult<Ctx> {
    let context = next(context.into_owned_request()).await?;

    Ok(context)
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

#[middleware_fn]
async fn not_found_404(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.body = Body::from(
        "<!DOCTYPE html>
    <html lang=\"en\">
    <head>
    <meta charset=\"utf-8\">
    <title>404 Not Found</title>
    </head>
    <body>
    <h1>404 Not Found</h1>
    </body>
    </html>",
    );
    context.content_type("text/html");
    context.status(404);

    Ok(context)
}

#[middleware_fn]
async fn post_new_todo(context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    match context.get_body().await {
        Ok((body, mut context)) => {
            let form_data = parse_form_data(body);
            // Because of the question marks, the closure should return false as soon as one of the
            // hashmap lookups returns None.
            let success = (|| {
                // Extract body data.
                let title = form_data.get("title")?;
                let due_date = form_data.get("due-date")?;
                let start_date = form_data.get("start-date")?;

                // Get todo list struct.
                let todos = context.extra.todos.clone();
                let mut todos = todos.write().unwrap();
                let new = IncomingTodo {
                    title: title.to_string(),
                    startable: NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
                        .unwrap_or_else(|_| NaiveDate::from_ymd(1970, 1, 1)),
                    due: NaiveDate::parse_from_str(due_date, "%Y-%m-%d")
                        .unwrap_or_else(|_| NaiveDate::from_ymd(1970, 1, 1)),
                };
                todos.add(new);

                Some(())
            })()
            .is_some();

            return if success {
                context.redirect("/index.html");
                Ok(context)
            } else {
                context.body = Body::from("request error");
                Ok(context)
            };
        }
        Err(e) => panic!("unrecoverable error: {:?}", e),
    }
}

fn parse_form_data(body: String) -> HashMap<String, String> {
    let mut form_hash = HashMap::new();

    {
        for query_piece in body.split('&') {
            let mut query_iterator = query_piece.split('=');
            let key = query_iterator.next().unwrap().to_owned();

            match query_iterator.next() {
                Some(val) => form_hash.insert(key, val.to_owned()),
                None => form_hash.insert(key, "true".to_owned()),
            };
        }
    }

    form_hash
}

#[middleware_fn]
async fn post_edit_todo(context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    if let Ok((body, mut context)) = context.get_body().await {
        let form_data = parse_form_data(body);
        // Because of the question marks, the closure should return false as soon as one of the
        // hashmap lookups returns None.
        let success = (|| {
            // Extract body data, URL params.
            let title = form_data.get("title")?;
            let due_date = form_data.get("due-date")?;
            let start_date = form_data.get("start-date")?;
            let id_string = context.query_params.get("id")?;
            let id = Uuid::parse_str(&id_string).ok()?;

            // Get todo list struct.
            let todos = context.extra.todos.clone();
            let mut todos = todos.write().unwrap();

            if let Some(existing) = todos.get(id) {
                let updated = Todo {
                    id: existing.id,
                    title: title.to_string(),
                    complete: existing.complete,
                    startable: NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
                        .unwrap_or_else(|_| NaiveDate::from_ymd(1970, 1, 1)),
                    due: NaiveDate::parse_from_str(due_date, "%Y-%m-%d")
                        .unwrap_or_else(|_| NaiveDate::from_ymd(1970, 1, 1)),
                };
                todos.update(updated);
            }
            Some(())
        })()
        .is_some();

        return if success {
            context.redirect("/index.html");
            Ok(context)
        } else {
            context.body = Body::from("request error");
            Ok(context)
        };
    } else {
        panic!("unrecoverable error")
    }
}

#[middleware_fn]
async fn get_new_todo(
    mut request_context: Ctx,
    _next: MiddlewareNext<Ctx>,
) -> MiddlewareResult<Ctx> {
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

#[middleware_fn]
async fn get_edit_todo(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let success = (|| {
        let todos = context.extra.todos.clone();
        let mut todos = todos.write().unwrap();

        let id = context.query_params.get("id")?;
        let uuid = Uuid::parse_str(&id).ok()?;
        if let Some(this_todo) = todos.get(uuid) {
            let mut tera_context = Context::new();
            tera_context.insert("todo", &this_todo);
            tera_context.insert("action", "Update");
            let tera = context.extra.tera.clone();
            let tera = tera.read().unwrap();
            let s = tera.render("detail.html.j2", &tera_context).ok()?;
            context.body = Body::from(s);
        } else {
            context.body = Body::from("unknown todo id");
        }
        Some(())
    })()
    .is_some();

    if success {
        Ok(context)
    } else {
        Err(ThrusterError {
            context,
            message: "invalid request".to_string(),
            status: 400,
            cause: None,
        })
    }
}

#[middleware_fn]
async fn post_complete_todo(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let success = (|| {
        let todos = context.extra.todos.clone();
        let mut todos = todos.write().unwrap();

        let id = context.query_params.get("id")?;
        let uuid = Uuid::parse_str(&id).ok()?;
        todos.toggle_completed(uuid);
        context.body = Body::from("{}");
        Some(())
    })()
    .is_some();

    if success {
        Ok(context)
    } else {
        Err(ThrusterError {
            context,
            message: "invalid request".to_string(),
            status: 400,
            cause: None,
        })
    }
}

#[middleware_fn]
async fn get_index(mut req_context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    println!("GET / HTTP/1.1");
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
    app.set404(async_middleware!(Ctx, [not_found_404]));
    // This doesn't appear to work. I asked in the Thruster discord.
    app.get("/", async_middleware!(Ctx, [get_index]));
    // But this one does, so I'm working around it temporarily until I get an answer.
    app.get("/index.html", async_middleware!(Ctx, [get_index]));
    app.post(
        "/new",
        async_middleware!(Ctx, [query_params, post_new_todo]),
    );
    app.get("/new", async_middleware!(Ctx, [get_new_todo]));
    app.post(
        "/edit",
        async_middleware!(Ctx, [query_params, post_edit_todo]),
    );
    app.get(
        "/edit",
        async_middleware!(Ctx, [query_params, get_edit_todo]),
    );
    app.post(
        "/complete",
        async_middleware!(Ctx, [query_params, post_complete_todo]),
    );
    app.get("/static/*", async_middleware!(Ctx, [file]));

    let server = HyperServer::new(app);
    server.start("0.0.0.0", 8080);
}
