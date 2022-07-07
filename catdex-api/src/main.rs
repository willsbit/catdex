#[macro_use]
extern crate diesel;

use actix_files::Files;
use actix_web::{http, web, App, HttpServer, Responder};
use serde::Serialize;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

mod models;
mod schema;
use self::models::*;
use self::schema::cats::dsl::*;
type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;


// async fn cats() -> impl Responder {
//     todo!()
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let manager =
        ConnectionManager::<PgConnection>::new(&database_url);
    
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create DB connection pool.");

    println!("Listening on port 8080");

    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            .service(
            web::scope("/api")
            .route("/cats", web::get().to(cats_endpoint)),
        )
            .service(
            Files::new("/", "static").show_files_listing(),
        )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}