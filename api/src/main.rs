use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponse};

#[get("/{id}/{name}/index.html")]
async fn index(web::Path((id, name)): web::Path<(u32, String)>) -> impl Responder {
    format!("Hello {}! id:{}", name, id)
}

#[get("/listMarkets")]
async fn list_markets(web::Path((id, name)): web::Path<(u32, String)>) -> impl Responder {
    format!("Hello {}! id:{}", "test", 10)
}

#[get("/dexInfo")]
async fn dex_info(web::Path((id, name)): web::Path<(u32, String)>) -> impl Responder {
    format!("Hello {}! id:{}", "test", 10)
}

#[get("/freezeBalance/{user}")]
async fn freeze_balance(web::Path((user)): web::Path<(String)>) -> impl Responder {
    format!("Hello {}! id:{}", user, 10)
}

async fn add_market(web::Path((contract_address)): web::Path<(String)>) -> HttpResponse {
    HttpResponse::Ok().body(contract_address)
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(index)
            .service(list_markets)
            .service(dex_info)
            .service(freeze_balance)
            .service(web::resource("/addMarket/{contract_address}").route(web::post().to(add_market)))
            .service(echo)
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
