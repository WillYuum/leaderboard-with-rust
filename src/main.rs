use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use rusqlite::{OpenFlags, Result};
use std::sync::{Arc, Mutex};

mod database;
use database::{create_table, get_all_leaderboard_data, update_leaderboard, LeaderboardEntry};
use rusqlite::Connection;
use serde::Serialize;

#[derive(Serialize)]
struct LeaderboardResponse {
    leaderboard: Vec<LeaderboardEntry>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let conn = Connection::open_with_flags(
        "test.db",
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    )
    .unwrap();

    create_table(&conn).unwrap();

    let shared_conn = Arc::new(Mutex::new(conn));

    let domain = unwrap_env_var("DOMAIN_NAME", "localhost");
    let port = unwrap_env_var("PORT", "8080");
    let addr = format!("{}:{}", domain, port);

    HttpServer::new(move || {
        let conn = shared_conn.clone();

        App::new()
            .app_data(web::Data::new(conn.clone())) // Wrap conn with Data::new()
            .route("/leaderboard", web::get().to(get_leaderboard))
        // .route("/leaderboard/{id}", web::post().to(update_score))
    })
    .bind(addr)?
    .run()
    .await?;

    Ok(())
}

#[get("/test")]
async fn greet() -> impl Responder {
    format!("Running leaderboard with rust!")
}

async fn get_leaderboard(data: web::Data<Arc<Mutex<Connection>>>) -> impl Responder {
    println!("GET /leaderboard");
    let conn = data.lock().unwrap();
    match get_all_leaderboard_data(&conn) {
        Ok(data) => {
            let response = LeaderboardResponse { leaderboard: data };
            HttpResponse::Ok().json(response)
        }
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
