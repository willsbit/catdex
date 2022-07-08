#[macro_use]
extern crate diesel;

use actix_files::Files;
use actix_web::{http, web, App, Error, HttpServer, Responder, HttpResponse, web::{Data}};
use serde::{Serialize, Deserialize};
use std::env;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use validator::Validate;
use validator_derive::Validate;

mod models;
mod schema;
use self::models::*;
use self::schema::cats::dsl::*;
type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;


/// Struct to hold the cat id's.
#[derive(Deserialize, Validate)]
struct CatEndpointPath {
    #[validate(range(min = 1, max = 150))]
    id: i32,
}

/// API endpoint to show details from a specific cat
async fn cat_endpoint(
    pool: web::Data<DbPool>,
    cat_id: web::Path<CatEndpointPath>
) -> Result<HttpResponse, Error> {

    cat_id
        .validate()
        .map_err(|_| HttpResponse::BadRequest().finish()).unwrap();


    let connection = pool.get()
        .expect("Can't get DB connection from pool");

    let cat_data = web::block(move || {
        cats.filter(id.eq(cat_id.id)).first::<Cat>(&connection)
    })
    .await
    .map_err(|_| HttpResponse::InternalServerError().finish()).unwrap().unwrap();

    Ok(HttpResponse::Ok().json(cat_data))
}

/// Endpoint to get 100 cats from the database
async fn cats_endpoint(
        pool: web::Data<DbPool>,
    ) -> Result<HttpResponse, Error> {
        let connection = pool.get()
            .expect("Can't get db connection from pool");
        let cats_data = web::block(move || {
            cats.limit(100).load::<Cat>(&connection)
            })
            .await
            .map_err(|_| HttpResponse::InternalServerError().finish()).ok().unwrap().unwrap();
        return Ok(HttpResponse::Ok().json(cats_data));
    }

/// Sets up a connecion pool with the Postgres instance
fn setup_database() -> DbPool {
    let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
    let manager =
            ConnectionManager::<PgConnection>::new(&database_url);
    
    r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create DB connection pool.")
}

/// Convenience function to avoid repeated code
fn api_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
        .route("/cats", web::get().to(cats_endpoint))
        .route("/cat/{id}", web::get().to(cat_endpoint))
    );
}

    #[actix_web::main]
    async fn main() -> std::io::Result<()> {
        
        let pool = setup_database();

        println!("Listening on port 8080");
        HttpServer::new(move || {
            App::new()
                .app_data(Data::new(pool.clone()))
                .configure(api_config)
                .service(
                    Files::new("/", "static").show_files_listing(),
                )
            })
            .bind("127.0.0.1:8080")?
            .run()
            .await
    }


    #[cfg(test)]
    mod tests {
        use super::*;
        use actix_web::{test, App};
        use actix_rt::*;

        #[actix_rt::test]
        async fn test_cats_endpoint_get() {
            let pool = setup_database();
            let mut app = test::init_service(
                App::new().app_data(Data::new(pool.clone())).configure(api_config)
            )
            .await;

            let req = test::TestRequest::get()
                .uri("/api/cats")
                .to_request();

            let resp = test::call_service(&mut app, req).await;

            assert!(resp.status().is_success());
        }
    }