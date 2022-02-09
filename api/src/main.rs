use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponse, error};
use serde::{Serialize,Deserialize};
use actix_cors::Cors;
use chemix_chain::{sign, listen_block};
use chemix_models::{api::{list_markets as list_markets2,MarketInfo}};

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
struct ChemixRespond {
    code : u8,
    msg : String,   //200 default success
    data : String,
}

fn respond_json(code: u8,msg: String,data: String) -> String{
    let respond = ChemixRespond {
        code,
        msg ,
        data,
    };
    serde_json::to_string(&respond).unwrap()
}

/***
* @api {get} /chemix/listMarkets list supported market
* @apiName listMarkets
* @apiGroup Exchange
*
* @apiSuccess {json} data market info
* @apiSuccessExample {json} Success-Response:
* {
*   "msg": "",
*   "data": [
*       {
*          "id":"WBTC-USDT",
*          "quote_token_address": "0x63210793010d03b04ddb61f8f219a8e7e40bcba668",
*          "base_token_address": "0x63bedfa1e1ea5891cb5f0819a7d16b7fe3aef5ddb0",
*          "quote_token_name": "BTC",
*          "base_token_name": "USDT",
*          "engine_address": "0x63210793010d03b04ddb61f8f219a8e7e40bcba668",
*       }
*   ],
*   "code": 200
* }
*@apiSampleRequest http://139.196.155.96:7010/chemix/listMarkets
 * */
#[get("/chemix/listMarkets")]
async fn list_markets(web::Path(()): web::Path<()>) -> impl Responder {
    let mut markets = Vec::<Markets>::new();
    let test1 = list_markets2();
    respond_json(
        200,
        "".to_string(),
        serde_json::to_string(&test1).unwrap()
    )

}


/***
* @api {get} /chemix/depth orderbook depth
* @apiName Depth
* @apiGroup Exchange
* @apiQuery {String} symbol market id
* @apiQuery {Number} limit  depth data size
* @apiSuccess {json} data  depth data
* @apiSuccessExample {json} Success-Response:
* {
*   "msg": "",
*   "data":{"asks":[
*               [50000.0,10.0001],
*               [40000.0,10.0002]
*        ],
*    "bids":[
*               [30000.0,10.0001],
*               [20000.0,10.0002]
*        ]
*   },
*   "code": 200
* }
*@apiSampleRequest http://139.196.155.96:7010/chemix/depth
 * */
#[derive(Deserialize,Serialize)]
struct DepthRequest {
    symbol: String,
    limit: u32,
}
#[get("/chemix/depth")]
async fn depth(web::Query(info): web::Query<DepthRequest>) -> String {
    format!("symbol222 {}, limit:{}", info.symbol,info.limit);
    respond_json(
        200,
        "".to_string(),
        serde_json::to_string(&info).unwrap()
    )
}




/***
* @api {get} /chemix/aggTrades recent trade
* @apiName AggTrades
* @apiGroup Exchange
* @apiQuery {String} symbol market id
* @apiQuery {Number} limit  trade data size
* @apiSuccess {json} data  depth data
* @apiSuccessExample {json} Success-Response:
* {
*   "msg": "",
*   "data":[
*        {"id":"BTC-USDT","price":1000.0,"amount":10.1,"taker_side":"buy","updated_at":1644287259123},
*        {"id":"BTC-USDT","price":1001.0,"amount":20.2,"taker_side":"sell","updated_at":1644287259123}
*   ]
*   "code": 200
* }
*@apiSampleRequest http://139.196.155.96:7010/chemix/aggTrade
 * */
#[derive(Deserialize,Serialize)]
struct AggTradesRequest {
    symbol: String,
    limit: u32,
}
#[get("/chemix/aggTrades")]
async fn agg_trades(web::Query(info): web::Query<AggTradesRequest>) -> impl Responder {
    respond_json(
        200,
        "".to_string(),
        serde_json::to_string(&info).unwrap()
    )
}


/***
* @api {get} /chemix/klines kline data
* @apiName klines
* @apiGroup Exchange
* @apiQuery {String} symbol market id
* @apiQuery {Number} limit  kline data size
* @apiQuery {Number} interval  kline type
* @apiSuccess {json} data  kline data
* @apiSuccessExample {json} Success-Response:
* {
*   "msg": "",
*   "data":[
*        {"id":"BTC-USDT","price":1000.0,"amount":10.1,"taker_side":"buy","updated_at":1644287259123},
*        {"id":"BTC-USDT","price":1001.0,"amount":20.2,"taker_side":"sell","updated_at":1644287259123}
*   ]
*   "code": 200
* }
*@apiSampleRequest http://139.196.155.96:7010/chemix/klines
 * */
#[derive(Deserialize,Serialize)]
struct KlinesRequest {
    symbol: String,
    limit: u32,
    interval: u32,
}
#[get("/chemix/klines")]
async fn klines(web::Query(info): web::Query<KlinesRequest>) -> impl Responder {
    respond_json(
        200,
        "".to_string(),
        serde_json::to_string(&info).unwrap()
    )
}



/***
* @api {post} /register   register WS connect
* @apiBody {Number} [user_id=1]
* @apiName ws_register
* @apiGroup WS
*
* @apiSuccess {json} result ws url
* @apiSuccessExample {json} Success-Response:
* {
*   "msg": "",
*   "data": {"url":"ws://139.196.155.96:7020/ws/a0d982449ae0489a84d8167289f690ec"},
*   "code": 200
* }
*
*@apiSampleRequest http://139.196.155.96:7020/register
 * */

/***
* @api {get} ----ws://139.196.155.96:7020/ws/373308c53a4545abaead65b04a857e2e    WS connect
* @apiName ws_subscribe
* @apiGroup WS
*
* @apiSuccess {json} depth price and volume in book
* @apiSuccess {json} aggTrade recent matched trade

* @apiSuccessExample {json} Success-Response:
*{"method": "SUBSCRIBE", "params": ["BTC-USDT@aggTrade"]}
*   [
*        {"id":"BTC-USDT","price":1000.0,"amount":10.1,"taker_side":"buy","updated_at":1644287259123},
*        {"id":"BTC-USDT","price":1001.0,"amount":20.2,"taker_side":"sell","updated_at":1644287259123}
*   ]
*{"method": "SUBSCRIBE", "params": ["BTC-USDT@depth"]}
*   {"asks":[
*               [1000.0,-10.0001],
*               [2000.0,10.0002]
*        ],
*    "bids":[
*               [1000.0,10.0001],
*               [2000.0,-10.0002]
*        ]
*   }
*{"method": "UNSUBSCRIBE", "params": ["BTC-USDT@depth"]}
* */

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
    //sign();
    //listen_block().await;
    let query_cfg = web::QueryConfig::default()
        .error_handler(|err, req| {
           error::InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
        });
    HttpServer::new(move || {
        App::new()//.app_data(query_cfg)
            .wrap(
            Cors::new()
                //.allowed_header("*")
                //.allowed_origin("*")
                //.allowed_origin("127.0.0.1")
                //.allowed_origin("192.168.1.139")
                //.send_wildcard()
            .allowed_methods(vec!["GET", "POST", "DELETE", "OPTIONS"])
            .max_age(3600)
            .finish())
            .service(index)
            .service(list_markets)
            .service(dex_info)
            .service(depth)
            .service(klines)
            .service(agg_trades)
            .service(freeze_balance)
            .service(web::resource("/addMarket/{contract_address}").route(web::post().to(add_market)))
            .service(echo)
            //.service(web::resource("/chemix/depth").route(web::get().to(depth)))
        })
        .bind("0.0.0.0:7010")?
        .run()
        .await
}
