use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponse};
use serde::Serialize;
use actix_cors::Cors;
use chemix_chain::{sign, listen_block};

/***
* @api {get} /user/:id Request User information
* @apiName GetUser
* @apiGroup User
*
* @apiParam {Number} id Users unique ID.
*
* @apiSuccess {String} firstname Firstname of the User.
 * @apiSuccess {String} lastname  Lastname of the User.
*
* @apiSuccessExample Success-Response:
 *     HTTP/1.1 200 OK
 *     {
 *       "firstname": "John",
 *       "lastname": "Doe"
 *     }
*
* @apiError UserNotFound The id of the User was not found.
*
* @apiErrorExample Error-Response:
 *     HTTP/1.1 404 Not Found
 *     {
 *       "error": "UserNotFound"
 *     }
 * */
#[get("/{id}/{name}/index.html")]
async fn index(web::Path((id, name)): web::Path<(u32, String)>) -> impl Responder {
    format!("Hello {}! id:{}", name, id)
}


#[derive(Serialize)]
struct Markets {
    quote_token_address: String,
    base_token_address: String,
    quote_token_name: String,
    base_token_name: String,
    engine_address: String,
}

#[derive(Serialize)]
struct MarketsRespond {
    success : bool,
    result : Vec<Markets>,
    error_code : u32,
}

/***
* @api {get} /chemix/listMarkets list supported market
* @apiName listMarkets
* @apiGroup Chemix
*
* @apiSuccess {json} result market info
     * @apiSuccessExample {json} Success-Response:
     * {
     *   "success": true,
     *   "result": [
     *       {
     *          "quote_token_address": "0x1",
     *          "base_token_address": "0x2",
     *          "quote_token_name": "BTC",
     *          "base_token_name": "USDT",
     *          "engine_address": "0x3",
     *       }
     *   ],
     *   "errorCode": 0
     * }
     *@apiSampleRequest /chemix/listMarkets

 * */
#[get("/chemix/listMarkets")]
async fn list_markets(web::Path(()): web::Path<()>) -> impl Responder {
    let mut markets = Vec::<Markets>::new();
    markets.push(Markets {
        quote_token_address: "0x1".to_string(),
        base_token_address: "0x2".to_string(),
        quote_token_name: "cETH".to_string(),
        base_token_name: "USDC".to_string(),
        engine_address: "0x3".to_string()
    });
    let respond = MarketsRespond {
        success : true,
        result: markets,
        error_code: 0
    };
    serde_json::to_string(&respond).unwrap()

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
    env_logger::init();
    sign();
    listen_block().await;
    HttpServer::new(move || {
        App::new().wrap(
            Cors::new()
                .allowed_header("*")
                .allowed_origin("*")
                //.allowed_origin("127.0.0.1")
                //.allowed_origin("192.168.1.139")
                //.send_wildcard()
            .allowed_methods(vec!["GET", "POST", "DELETE", "OPTIONS"])
            .max_age(3600)
            .finish())
        .service(index)
        .service(list_markets)
        .service(dex_info)
        .service(freeze_balance)
        .service(web::resource("/addMarket/{contract_address}").route(web::post().to(add_market)))
        .service(echo)
    })
        .bind("0.0.0.0:8000")?
        .run()
        .await
}
