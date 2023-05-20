use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use rusqlite::OpenFlags;
use std::sync::{Arc, Mutex};
use time::OffsetDateTime;

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
    dotenv::dotenv().ok();

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
            .route(
                "/leaderboard/get-highscore/{id}",
                web::get().to(get_user_highscore),
            )
            .route("/leaderboard", web::get().to(get_leaderboard))
            .route("/leaderboard/{id}", web::post().to(update_score))
            .route(
                "/leaderboard/{username}/{highscore}",
                web::post().to(create_new_user),
            )
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

async fn get_user_highscore(
    data: web::Data<Arc<Mutex<Connection>>>,
    path: web::Path<(String,)>,
) -> impl Responder {
    println!("GET /leaderboard/{}", path.0);
    let conn = data.lock().unwrap();
    match database::get_user_highscore(&conn, &path.0) {
        Ok(data) => {
            let response = data;
            HttpResponse::Ok().json(response)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn update_score(
    data: web::Data<Arc<Mutex<Connection>>>,
    path: web::Path<(i32,)>,
) -> impl Responder {
    println!("POST /leaderboard/{}", path.0);
    let conn = data.lock().unwrap();
    match update_leaderboard(&conn, path.0, 100) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn create_new_user(
    data: web::Data<Arc<Mutex<Connection>>>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let conn = data.lock().unwrap();

    let result = database::user_exists(&conn, &path.0).unwrap();

    match result {
        true => {
            println!("User already exists");
            return HttpResponse::Ok().finish();
        }
        false => {
            let time_stamp = OffsetDateTime::now_utc();
            println!("Creating user at time{}", OffsetDateTime::now_utc());
            println!("POST /leaderboard/{} {}", path.0, path.1);
            match database::add_new_user(&conn, &path.0, &path.1, &time_stamp.to_string()) {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
    }
}

fn unwrap_env_var(var_name: &str, fallback: &str) -> String {
    use std::env::var as env_var;

    env_var(var_name).unwrap_or_else(|err| {
        println!("{}: {}", var_name, err);
        fallback.to_string()
    })
}
