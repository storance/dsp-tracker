mod data;
mod error;
mod field;
mod game_save;
mod planet;
mod solar_system;
mod star;
mod utils;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use error::TrackerError;
use sqlx::postgres::{PgPool, PgPoolOptions};

const DEFAULT_LISTEN_PORT: u16 = 8080;

pub struct AppState {
    db: PgPool,
}

fn config(cfg: &mut web::ServiceConfig) {
    let scope = web::scope("/api/1")
        .configure(game_save::config)
        .configure(solar_system::config)
        .configure(star::config);
    cfg.service(scope);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let conn_str = std::env::var("DATABASE_URL").expect("Env var DATABASE_URL is required.");
    let cors_permissive = std::env::var("CORS_PERMISSIVE").map_or(false, |v| v.eq("true"));
    let listen_port = std::env::var("LISTEN_PORT").map_or(DEFAULT_LISTEN_PORT, |v| {
        str::parse(&v).expect("Env var LISTEN_PORT is invalid")
    });
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&conn_str)
        .await
        .expect("Failed to connect to the database");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run sql migrations");

    HttpServer::new(move || {
        let cors = if cors_permissive {
            Cors::permissive()
        } else {
            Cors::default()
        };
        App::new()
            .app_data(web::Data::new(AppState { db: pool.clone() }))
            .app_data(
                web::JsonConfig::default()
                    .error_handler(|err, _req| TrackerError::from(err).into()),
            )
            .app_data(
                web::QueryConfig::default()
                    .error_handler(|err, _req| TrackerError::from(err).into()),
            )
            .app_data(
                web::PathConfig::default()
                    .error_handler(|err, _req| TrackerError::from(err).into()),
            )
            .configure(config)
            .wrap(cors)
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", listen_port))?
    .run()
    .await
}
