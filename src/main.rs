use std::collections::HashMap;

use actix_web::{
    middleware,
    web::{self, redirect, Redirect},
    App, FromRequest, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use actix_web_lab::respond::Html;
use askama::Template;
use askama_actix::TemplateToResponse;

mod rfc_parser;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

async fn index() -> impl Responder {
    IndexTemplate {}
}

#[derive(Template)]
#[template(path = "rfc_view.html")]
struct RFCViewTemplate {
    rfc_data: rfc_parser::RFCData,
}

async fn rfc(req: HttpRequest, query: web::Query<HashMap<String, String>>) -> HttpResponse {
    if let Some(number) = query.get("number") {
        if let Ok(number) = number.parse::<u32>() {
            if let Ok(data) = rfc_parser::RFCData::new(number).await {
                println!("rfc_data is {:?}", data);
                return RFCViewTemplate { rfc_data: data }.to_response();
            }
        }
    }
    Redirect::to("/").respond_to(&req).map_into_boxed_body()
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/rfc").route(web::get().to(rfc)))
    })
    .bind(("127.0.0.1", 3030))?
    .run()
    .await
}
