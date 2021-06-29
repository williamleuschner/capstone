extern crate futures;
extern crate serde;
extern crate thruster;

mod context;

use context::{generate_context, Ctx};
use futures::future; // , Future};
use serde::{Deserialize, Serialize};
use std::boxed::Box;
use thruster::{App, MiddlewareChain, MiddlewareReturnValue};
// use thruster::{BasicContext as Ctx, Request};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
struct Todo {
    title: String,
    complete: bool,
    startable: DateTime<Utc>,
    due: DateTime<Utc>,
}

fn not_found_404(context: Ctx, _chain: &MiddlewareChain<Ctx>) -> MiddlewareReturnValue<Ctx> {
    let mut context = Ctx::new(context);

    context.body = "<!DOCTYPE html>
    <html lang=\"en\">
    <head>
    <meta charset=\"utf-8\">
    <title>404 Not Found</title>
    </head>
    <body>
    <h1>404 Not Found</h1>
    </body>
    </html>"
        .to_owned();
    context.set_header("Content-Type".to_owned(), "text/html".to_owned());
    context.status_code = 404;

    Box::new(future::ok(context))
}

fn greeting(context: Ctx, _chain: &MiddlewareChain<Ctx>) -> MiddlewareReturnValue<Ctx> {
    let mut context = Ctx::new(context);

    println!("{:?}", context.headers);
    if let Some(name) = context.params.get("name") {
        context.body = format!("Hello, {}!", name);
    } else {
        context.body = "Hello, World!".to_owned();
    }

    context.set_header("Server".to_owned(), "thruster".to_owned());
    context.set_header("Content-Type".to_owned(), "text/plain".to_owned());

    Box::new(future::ok(context))
}

fn create(context: Ctx, _chain: &MiddlewareChain<Ctx>) -> MiddlewareReturnValue<Ctx> {
    let mut context = Ctx::new(context);

    let title = context.request_body.clone();
    let date = Utc::now();
    let new = Todo {
        title: title,
        complete: false,
        startable: date,
        due: date,
    };
    let new_json = serde_json::to_string(&new);

    if let Ok(result) = new_json {
        context.body = result;
        context.set_header("Content-Type".to_owned(), "application/json".to_owned());
    } else {
        context.body = "Whoops".to_owned();
        context.set_header("Content-Type".to_owned(), "text/plain".to_owned());
        context.status_code = 500;
    }

    Box::new(future::ok(context))
}

fn main() {
    println!("Starting server...");

    let mut app = App::<Ctx>::create(generate_context);

    // _app.use_middleware("/", profiling);

    app.get("/greeting/:name", vec![greeting]);
    app.post("/todo/create", vec![create]);
    app.set404(vec![not_found_404]);

    App::start(app, "0.0.0.0".to_string(), "8080".to_string());
}
