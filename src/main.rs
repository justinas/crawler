use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use actix_web::{
    get, post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use serde::Deserialize;

use worker::Storage;

mod worker;

const DOC: &str = r#"
GET /

    this page

POST /crawl

    start crawling

    > {"domain": "http://justinas.org"}
"#;

#[derive(Deserialize)]
struct CrawlRequest {
    domain: url::Url,
}

#[get("/")]
async fn index() -> impl Responder {
    DOC
}

#[post("/crawl")]
async fn crawl(storage: Data<Arc<Storage>>, request: web::Json<CrawlRequest>) -> impl Responder {
    let domain = request.0.domain;
    if !matches!(domain.host(), Some(url::Host::Domain(_)))
        || domain.host_str() == Some("localhost")
    {
        return HttpResponse::BadRequest();
    }
    tokio::spawn(worker::crawl(storage.get_ref().clone(), domain));
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let storage: Arc<Storage> = Arc::new(RwLock::new(HashMap::<String, HashSet<String>>::new()));
    Ok(HttpServer::new(move || {
        App::new()
            .data(storage.clone())
            .service(index)
            .service(crawl)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?)
}
