use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use rusqlite::Result;

mod database;
use database::{create_table, get_all_leaderboard_data, update_leaderboard, LeaderboardEntry};
use rusqlite::Connection;

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

#[get("/test")]
async fn greet() -> impl Responder {
    format!("Running leaderboard with rust!")
}

async fn get_leaderboard(data: web::Data<Connection>) -> impl Responder {
    println!("GET /leaderboard");
    match get_all_leaderboard_data(&data) {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// async fn update_score(
//     data: web::Data<Connection>,
//     web::Path(id): web::Path<i32>,
//     web::Json(score): web::Json<i32>,
// ) -> impl Responder {
//     println!("POST /leaderboard/{}", id);
//     match update_leaderboard(&data, id, score) {
//         Ok(_) => HttpResponse::Ok().finish(),
//         Err(_) => HttpResponse::InternalServerError().finish(),
//     }
// }

fn unwrap_env_var(var_name: &str, fallback: &str) -> String {
    use std::env::var as env_var;

    env_var(var_name).unwrap_or_else(|err| {
        println!("{}: {}", var_name, err);
        fallback.to_string()
    })
}
