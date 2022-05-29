use actix_web::{
    get, guard,
    http::header::ContentType,
    post,
    web,
    App, Error, HttpResponse, HttpServer, Responder, rt::time::sleep
};
use futures::stream::{repeat_with, StreamExt};
use serde::{Deserialize, Serialize};
use std::{sync::atomic::{self, Ordering}, net::Ipv4Addr, time::Duration};

struct AppState {
    app_name: String,
    counter: atomic::AtomicUsize,
}

#[derive(Serialize)]
enum Gender {
    Male,
    Female,
}

#[derive(Serialize)]
struct User {
    name: String,
    age: u16,
    gender: Gender,
}

#[get("/")]
async fn hello(data: web::Data<AppState>) -> impl Responder {
    let counter = data.counter.fetch_add(1, Ordering::Relaxed) + 1;
    sleep(Duration::from_secs(5)).await;
    return HttpResponse::Ok().body(format!(
        "Hello, this is {} and it has been visited {} times!",
        data.app_name,
        counter
    ));
}

#[derive(Deserialize)]
struct Info {
    limit: usize,
}

#[post("/echo")]
async fn echo(req_body: String, info: web::Query<Info>) -> impl Responder {
    let body = repeat_with(move || web::Bytes::from(req_body.clone()));
    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .streaming(body.take(info.limit).map(|f| Ok::<_, Error>(f)))
}

async fn manual_hello() -> impl Responder {
    let user = User {
        name: "Sanndy".to_string(),
        age: 10,
        gender: Gender::Female,
    };
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(serde_json::to_string(&user).unwrap_or_else(|_| "{}".to_string()))
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/app")
            .guard(guard::Header("Host", "localhost"))
            .route(web::get().to(|| async { HttpResponse::Ok().body("app") }))
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        app_name: String::from("Sanndy App"),
        counter: atomic::AtomicUsize::new(0),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(hello)
            .service(echo)
            .configure(config)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind((Ipv4Addr::new(127, 0, 0, 1), 8080))?
    .run()
    .await
}
