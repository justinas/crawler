use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

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
async fn crawl(request: web::Json<CrawlRequest>) -> impl Responder {
    let domain = request.0.domain;
    if !matches!(domain.host(), Some(url::Host::Domain(_)))
        || domain.host_str() == Some("localhost")
    {
        return HttpResponse::BadRequest();
    }
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    Ok(HttpServer::new(|| App::new().service(index).service(crawl))
        .bind("127.0.0.1:8080")?
        .run()
        .await?)
}
