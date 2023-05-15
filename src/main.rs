use actix_web::{get, App, HttpServer, Responder};

#[get("/test")]
async fn greet() -> impl Responder {
    format!("Running leaderboard with rust!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let domain = unwrap_env_var("DOMAIN_NAME", "localhost");
    let port = unwrap_env_var("PORT", "8080");

    let addr = format!("{}:{}", domain, port);

    println!("Running server on: {}", addr);

    HttpServer::new(|| App::new().service(greet))
        .bind(&addr)?
        .run()
        .await
}

fn unwrap_env_var(var_name: &str, fallback: &str) -> String {
    use std::env::var as env_var;

    env_var(var_name).unwrap_or_else(|err| {
        println!("{}: {}", var_name, err);
        fallback.to_string()
    })
}
