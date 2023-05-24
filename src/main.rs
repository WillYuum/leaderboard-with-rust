use actix_web::{web, App, HttpResponse, HttpServer, Responder};
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

    println!("Running server on http://{}", &addr);

    HttpServer::new(move || {
        let conn = shared_conn.clone();

        App::new()
            .app_data(web::Data::new(conn.clone())) // Wrap conn with Data::new()
            .route(
                "/add-to-leaderboard/{username}/{highscore}",
                web::post().to(create_new_user),
            )
            .route("/get-leaderboard", web::get().to(get_leaderboard))
            .route("/get-leaderboard/{id}", web::get().to(get_user_highscore))
            .route(
                "/udpate-leaderboard/{id}/{newScore}",
                web::post().to(update_score),
            )
    })
    .bind(&addr)?
    .run()
    .await?;

    Ok(())
}

async fn get_leaderboard(data: web::Data<Arc<Mutex<Connection>>>) -> impl Responder {
    println!("GET /leaderboard ALL");
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
    path: web::Path<String>,
) -> impl Responder {
    struct GetHighScoreQuery {
        id: String,
    }

    let query: String = path.into_inner();
    let new_user_query = GetHighScoreQuery { id: query };

    let conn = data.lock().unwrap();
    match database::get_user_highscore(&conn, &new_user_query.id) {
        Ok(data) => {
            println!("GET /leaderboard/ highscore: {}", &data);

            let response = data;
            HttpResponse::Ok().json(response)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn update_score(
    data: web::Data<Arc<Mutex<Connection>>>,
    path: web::Path<(i32, i32)>,
) -> impl Responder {
    struct UpdateScoreQuery {
        id: i32,
        new_highscore: i32,
    }

    let query: (i32, i32) = path.into_inner();
    let new_user_query = UpdateScoreQuery {
        id: query.0,
        new_highscore: query.1,
    };

    let conn = data.lock().unwrap();
    let result = update_leaderboard(&conn, new_user_query.id, new_user_query.new_highscore);
    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn create_new_user(
    data: web::Data<Arc<Mutex<Connection>>>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    struct NewUserQuery {
        username: String,
        highscore: String,
    }

    let query = path.into_inner();
    let new_user_query = NewUserQuery {
        username: query.0,
        highscore: query.1,
    };

    let time_stamp = OffsetDateTime::now_utc();

    let conn = data.lock().unwrap();
    let result = database::add_new_user(
        &conn,
        &new_user_query.username,
        &new_user_query.highscore,
        &time_stamp.to_string(),
    );

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

fn unwrap_env_var(var_name: &str, fallback: &str) -> String {
    use std::env::var as env_var;

    env_var(var_name).unwrap_or_else(|err| {
        println!("{}: {}", var_name, err);
        fallback.to_string()
    })
}
