use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use actix_web::{
    get, post,
    web::{self, Data},
    App, Either, HttpResponse, HttpServer, Responder,
};
use serde::{Deserialize, Serialize};

use worker::Storage;

mod worker;

const DOC: &str = r#"
GET /

    this page

POST /crawl

    start crawling

    > {"domain": "http://example.com"}

GET /domains

    get the list of domains

    < ["http://example.com"]

GET /domains/example.com

    get crawled links for example.com

    < {"count":2,"urls":["http://example.com/foo","http://bar.com"]}
"#;

#[derive(Deserialize)]
struct CrawlRequest {
    domain: url::Url,
}

#[derive(Serialize)]
struct DomainResponse {
    count: usize,
    urls: Vec<String>,
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

#[get("/domains")]
async fn domains(storage: Data<Arc<Storage>>) -> impl Responder {
    web::Json(storage.read().unwrap().keys().cloned().collect::<Vec<_>>())
}

#[get("/domains/{domain}")]
async fn domain_(storage: Data<Arc<Storage>>, path: web::Path<String>) -> impl Responder {
    match storage.read().unwrap().get(&path.into_inner()) {
        Some(s) => Either::Left(web::Json(DomainResponse {
            count: s.len(),
            urls: s.iter().cloned().collect(),
        })),
        None => Either::Right(HttpResponse::NotFound()),
    }
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
            .service(domains)
            .service(domain_)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?)
}
