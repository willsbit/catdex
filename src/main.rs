#[macro_use]
extern crate diesel;

use actix_files::Files;
use actix_web::{http, web, App, Error, HttpResponse, HttpServer};
use awmp::Parts;
use std::collections::HashMap;
use std::env;
use actix_web::web::Data;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use serde::{Serialize};

use self::models::*;
use handlebars::Handlebars;
mod models;
mod schema;
use self::schema::cats::dsl::*;


type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Serialize)]
struct IndexTemplateData {
    project_name: String,
    cats: Vec<self::models::Cat>
}

async fn add(
    hb: web::Data<Handlebars<'_>>,
) -> Result<HttpResponse, Error> {
    let body = hb.render("add", &{}).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

async fn add_cat_form(
    pool: web::Data<DbPool>,
    mut parts: Parts,
) -> Result<HttpResponse, Error> {
    let file_path = parts
        .files
        .take("image")
        .pop()
        .and_then(|f| f.persist_in("./static/image").ok())
        .unwrap_or_default();

    let text_fields: HashMap<_, _> = 
        parts.texts.as_pairs().into_iter().collect();

    let connection = pool.get().expect("Can't get a DB connection from pool.");

    let new_cat = NewCat {
        name: text_fields.get("name").unwrap().to_string(),
        image_path: file_path.to_string_lossy().to_string()
    };

    web::block(move || {
        diesel::insert_into(cats)
        .values(&new_cat)
        .execute(&connection)
        })
        .await
        .map_err(|_| HttpResponse::InternalServerError().finish()).unwrap();

    // redirect sucess with 303 See Other status code
    Ok(HttpResponse::SeeOther()
        .header(http::header::LOCATION, "/")
        .finish())

}


async fn index(
    hb: web::Data<Handlebars<'_>>,
    pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let connection = pool.get().expect("Can't get a DB connection from pool");
    let cats_data = web::block(move || cats.limit(100).load::<Cat>(&connection))
        .await
        .map_err(|_| {
            HttpResponse::InternalServerError().finish()
        }).unwrap().unwrap(); //there's gotta be a better way to do this

    let data = IndexTemplateData {
        project_name: "Catdex".to_string(),
        cats: cats_data
    };

    let body = hb.render("index", &data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

/// Show a cat details page, with the url pattern /cat/<id>
async fn cat(
    hb: web::Data<Handlebars<'_>>,
    pool: web::Data<DbPool>,
    cat_id: web::Path<i32>
) -> Result<HttpResponse, Error> {
    let connection = pool.get()
        .expect("Can't get a DB connection from pool");
    
    let cat_data = web::block(move || {
        cats.filter(id.eq(cat_id.into_inner()))
        .first::<Cat>(&connection)
    })
    .await
    .map_err(|_| HttpResponse::InternalServerError().finish()).unwrap().unwrap();

    let body = hb.render("cat", &cat_data).unwrap();

    Ok(HttpResponse::Ok().body(body))
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".html", "./static/")
        .unwrap();

    let handlebars_ref = web::Data::new(handlebars);

    // Set up the database connection pool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager =
        ConnectionManager::<PgConnection>::new(&database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create DB connection pool");

    println!("Listening on port 8080");
    HttpServer::new(move || {
        App::new()
            .app_data(handlebars_ref.clone())
            .app_data(Data::new(pool.clone()))
            .service(
                Files::new("/static", "static")
                    .show_files_listing(), // handy for debugging, but should be turned off in production
            )
            .route("/", web::get().to(index))
            .route("/add", web::get().to(add))
            .route("/cat/{id}", web::get().to(cat))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}