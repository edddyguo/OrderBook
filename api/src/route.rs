mod depth;
mod kline;
mod market;
mod order;
mod trade;

use actix_cors::Cors;
use actix_web::{error, get, post, web, App, HttpResponse, HttpServer, Responder};
use std::env;
use log::info;

use chemix_models::api::list_markets as list_markets2;
use chemix_models::order::{EngineOrderTmp2, list_available_orders, list_users_orders, Status};
use chemix_models::trade::list_trades;
use chemix_utils::time::time2unix;
use serde::{Deserialize, Serialize};
use chemix_models::order::Side::{Buy, Sell};
use chemix_utils::env::EnvConf;
use chemix_utils::math::u256_to_f64;


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

#[derive(Deserialize, Serialize)]
struct DexProfile {
    cumulativeTVL: f64,
    cumulativeTransactions: u32,
    cumulativeTraders: u32,
    numberOfTraders: u32,
    tradingVolume: f64,
    numberOfTransactions: u32,
    TVL: f64,
    tradingPairs: u8,
    price: f64,
}

#[derive(Serialize)]
struct ChemixRespond {
    code: u8,
    msg: String,
    //200 default success
    data: String,
}


#[derive(Deserialize, Serialize)]
struct DexInfo {
    engine_address: String,
    vault_address: String,
    proxy_address: String,
}

fn respond_json(code: u8, msg: String, data: String) -> String {
    let respond = ChemixRespond { code, msg, data };
    serde_json::to_string(&respond).unwrap()
}


/***
* @api {get} /chemix/dexInfo dex_info
* @apiName dex_info
* @apiGroup Exchange
*
* @apiSuccess {json} data dex_info
* @apiSuccessExample {json} Success-Response:
*{
*    "code": 200,
*    "msg": "",
*    "data": "{\"engine_address\":\"0xde49632Eb0416C5cC159d707B4DE0d4724427999\",\"vault_address\":\"0xC94393A080Df85190541D45d90769aB8D19f30cE\",\"proxy_address\":\"0xA1351C4e528c705e5817c0dd242C1b9dFccfD7d4\"}"
*}
*@apiSampleRequest http://139.196.155.96:7010/chemix/dexInfo
 * */

#[get("/chemix/dexInfo")]
async fn dex_info(web::Path(()): web::Path<()>) -> impl Responder {
    let engine = chemix_utils::env::CONF.chemix_main.to_owned().unwrap();
    let vault = chemix_utils::env::CONF.chemix_vault.to_owned().unwrap();
    let proxy = chemix_utils::env::CONF.chemix_token_proxy.to_owned().unwrap();

    let dex_info = DexInfo {
        engine_address: engine.into_string().unwrap(),
        vault_address: vault.into_string().unwrap(),
        proxy_address: proxy.into_string().unwrap(),
    };
    respond_json(200, "".to_string(), serde_json::to_string(&dex_info).unwrap())
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
    let _markets = Vec::<Markets>::new();
    let test1 = list_markets2();
    respond_json(200, "".to_string(), serde_json::to_string(&test1).unwrap())
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
#[derive(Deserialize, Serialize)]
struct DepthRequest {
    symbol: String,
    limit: u32,
}

#[get("/chemix/depth")]
async fn dex_depth(web::Query(info): web::Query<DepthRequest>) -> String {
    format!("symbol222 {}, limit:{}", info.symbol, info.limit);
    let base_decimal = 18u32;
    let quote_decimal = 15u32;
    let available_buy_orders = list_available_orders("BTC-USDT", Buy);
    let available_sell_orders = list_available_orders("BTC-USDT", Sell);

    info!("0001__{:?}",available_buy_orders);
    info!("0002__{:?}",available_sell_orders);
    let mut asks = Vec::<(f64, f64)>::new();
    let mut bids = Vec::<(f64, f64)>::new();

    'buy_orders: for available_buy_order in available_buy_orders {
        'asks: for mut bid in bids.clone() {
            if u256_to_f64(available_buy_order.price, quote_decimal) == bid.0 {
                bid.1 += u256_to_f64(available_buy_order.amount, base_decimal);
                continue 'buy_orders;
            }
        }
        if bids.len() as u32 == info.limit {
            break 'buy_orders;
        }
        bids.push((u256_to_f64(available_buy_order.price, quote_decimal), u256_to_f64(available_buy_order.amount, base_decimal)));
    }

    'sell_orders: for available_sell_order in available_sell_orders {
        'bids: for mut ask in asks.clone() {
            if u256_to_f64(available_sell_order.price, quote_decimal) == ask.0 {
                ask.1 += u256_to_f64(available_sell_order.amount, base_decimal);
                continue 'sell_orders;
            }
        }
        if asks.len() as u32 == info.limit {
            break 'sell_orders;
        }
        asks.push((u256_to_f64(available_sell_order.price, quote_decimal), u256_to_f64(available_sell_order.amount, base_decimal)));
    }

    let mut depth_data = depth::Depth { asks, bids };
    depth_data.sort();
    respond_json(
        200,
        "".to_string(),
        serde_json::to_string(&depth_data).unwrap(),
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
*@apiSampleRequest http://139.196.155.96:7010/chemix/aggTrades
 * */
#[derive(Deserialize, Serialize)]
struct AggTradesRequest {
    symbol: String,
    limit: u32,
}

#[get("/chemix/aggTrades")]
async fn agg_trades(web::Query(info): web::Query<AggTradesRequest>) -> impl Responder {
    let base_decimal = 18u32;
    let quote_decimal = 15u32;
    let trades = list_trades(None, Some("BTC-USDT".to_string()), info.limit)
        .iter()
        .map(|x| trade::Trade {
            id: x.id.clone(),
            price: u256_to_f64(x.price, quote_decimal),
            amount: u256_to_f64(x.amount, base_decimal),
            height: 12345u32,
            taker_side: x.taker_side.clone(),
            updated_at: time2unix(x.created_at.clone()),
        })
        .collect::<Vec<trade::Trade>>();

    respond_json(200, "".to_string(), serde_json::to_string(&trades).unwrap())
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
#[derive(Deserialize, Serialize)]
struct KlinesRequest {
    symbol: String,
    limit: u32,
    interval: u32,
}

#[get("/chemix/klines")]
async fn klines(web::Query(info): web::Query<KlinesRequest>) -> impl Responder {
    respond_json(200, "".to_string(), serde_json::to_string(&info).unwrap())
}

/***
* @api {get} ----ws://139.196.155.96:7020/chemix   WS connect
* @apiName ws_subscribe
* @apiGroup WS
*
* @apiSuccess {json} depth price and volume in book
* @apiSuccess {json} aggTrade recent matched trade

* @apiSuccessExample {json} Success-Response:
*{"method": "SUBSCRIBE","params":{"channel":["BTC-USDT@aggTrade"],"hash":""}}
*   [
*        {"id":"BTC-USDT","price":1000.0,"amount":10.1,"taker_side":"buy","updated_at":1644287259123},
*        {"id":"BTC-USDT","price":1001.0,"amount":20.2,"taker_side":"sell","updated_at":1644287259123}
*   ]
*
*
*{"method": "SUBSCRIBE","params":{"channel":["BTC-USDT@depth"],"hash":""}}
*   {"asks":[
*               [1000.0,-10.0001],
*               [2000.0,10.0002]
*        ],
*    "bids":[
*               [1000.0,10.0001],
*               [2000.0,-10.0002]
*        ]
*   }
*
*
*{"method": "PING","params":{"channel":[],"hash":""}}
*   {"channel":"","method":"PONG","data":""}
* */


/***
* @api {get} /dashBoard/profile dex profile info
* @apiName profile
* @apiGroup dashBoard
*
* @apiSuccess {json} data profile info
* @apiSuccessExample {json} Success-Response:
* {
*	"code": 200,
*	"msg": "",
*	"data": "{\"cumulativeTVL\":0.0,\"cumulativeTransactions\":0,\"cumulativeTraders\":0,\"numberOfTraders\":0,\"tradingVolume\":0.0,\"numberOfTransactions\":0,\"TVL\":0.0,\"tradingPairs\":0,\"price\":0.0}"
* }
*@apiSampleRequest http://139.196.155.96:7010/dashBoard/profile
 * */
#[get("/dashBoard/profile")]
async fn dex_profile() -> impl Responder {
    let profile = DexProfile {
        cumulativeTVL: 0.0,
        cumulativeTransactions: 0,
        cumulativeTraders: 0,
        numberOfTraders: 0,
        tradingVolume: 0.0,
        numberOfTransactions: 0,
        TVL: 0.0,
        tradingPairs: 0,
        price: 0.0,
    };
    respond_json(200, "".to_string(), serde_json::to_string(&profile).unwrap())
}

#[get("/freezeBalance/{user}")]
async fn freeze_balance(web::Path(user): web::Path<String>) -> impl Responder {
    format!("Hello {}! id:{}", user, 10)
}

async fn add_market(web::Path(contract_address): web::Path<String>) -> HttpResponse {
    HttpResponse::Ok().body(contract_address)
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}


/***
* @api {get} /chemix/listOrders listOrders
* @apiName listOrders
* @apiGroup Exchange
* @apiQuery {String} account user
* @apiQuery {Number} limit  trade data size
* @apiSuccess {json} data  current available orders
* @apiSuccessExample {json} Success-Response:
* {
*   "msg": "",
*   "data":[
*        {"id":"BTC-USDT","price":1000.0,"amount":10.1,"taker_side":"buy","updated_at":1644287259123},
*        {"id":"BTC-USDT","price":1001.0,"amount":20.2,"taker_side":"sell","updated_at":1644287259123}
*   ]
*   "code": 200
* }
*@apiSampleRequest http://139.196.155.96:7010/chemix/listOrders
 * */
#[derive(Deserialize, Serialize, Debug)]
struct ListOrdersRequest {
    account: String,
    limit: u32,
}

//todo: 所有的数据库都加上numPows字段，在后models里处理
#[get("/chemix/listOrders")]
async fn list_orders(web::Query(info): web::Query<ListOrdersRequest>) -> impl Responder {
    let base_decimal = 18u32;
    let quote_decimal = 15u32;
    let account = info.account.clone().to_lowercase();
    let orders = list_users_orders(account.as_str(), Status::from("pending"), Status::from("partial_filled"), info.limit);
    let orders = orders.iter().map(|x| {
        EngineOrderTmp2 {
            id: "BTC-USDT".to_string(),
            index: x.index.to_string(),
            account: x.account.clone(),
            price: u256_to_f64(x.price, quote_decimal),
            amount: u256_to_f64(x.amount, base_decimal),
            side: x.side.as_str().to_string(),
            status: x.status.as_str().to_string(),
            created_at: "".to_string(),
        }
    }).collect::<Vec<EngineOrderTmp2>>();
    respond_json(200, "".to_string(), serde_json::to_string(&orders).unwrap())
}


/***
* @api {get} /chemix/recentTrades recentTrades
* @apiName recentTrades
* @apiGroup Exchange
* @apiQuery {String} account user
* @apiQuery {Number} limit  trade data size
* @apiSuccess {json} data  recentTrades
* @apiSuccessExample {json} Success-Response:
* {
*   "msg": "",
*   "data":[
*        {"id":"BTC-USDT","price":1000.0,"amount":10.1,"taker_side":"buy","updated_at":1644287259123},
*        {"id":"BTC-USDT","price":1001.0,"amount":20.2,"taker_side":"sell","updated_at":1644287259123}
*   ]
*   "code": 200
* }
*@apiSampleRequest http://139.196.155.96:7010/chemix/recentTrades
 * */
#[derive(Deserialize, Serialize, Debug)]
struct RecentTradesRequest {
    account: String,
    limit: u32,
}

#[get("/chemix/recentTrades")]
async fn recent_trades(web::Query(info): web::Query<RecentTradesRequest>) -> impl Responder {
    let base_decimal = 18u32;
    let quote_decimal = 15u32;
    let account = info.account.clone().to_lowercase();
    let trades = list_trades(Some(account.clone()), None, info.limit);
    let trades = trades.iter().map(|x| {
        let side = if account == x.taker {
            x.taker_side.clone()
        }else {
            x.taker_side.contrary()
        };
        trade::Trade {
            id: x.id.clone(),
            price: u256_to_f64(x.price, quote_decimal),
            amount: u256_to_f64(x.amount, base_decimal),
            height: 12345u32,
            // fixme: maybe side?
            taker_side: side,
            updated_at: time2unix(x.created_at.clone()),
        }
    }).collect::<Vec<trade::Trade>>();
    respond_json(200, "".to_string(), serde_json::to_string(&trades).unwrap())
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    //sign();
    //listen_block().await;
    let _query_cfg = web::QueryConfig::default().error_handler(|err, _req| {
        error::InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
    });

    let port = match env::var_os("API_PORT") {
        None => 7010u32,
        Some(mist_mode) => mist_mode.into_string().unwrap().parse::<u32>().unwrap(),
    };
    let service = format!("0.0.0.0:{}", port);

    HttpServer::new(move || {
        App::new() //.app_data(query_cfg)
            .wrap(
                Cors::new()
                    //.allowed_header("*")
                    //.allowed_origin("*")
                    //.allowed_origin("127.0.0.1")
                    //.allowed_origin("192.168.1.139")
                    //.send_wildcard()
                    .allowed_methods(vec!["GET", "POST", "DELETE", "OPTIONS"])
                    .max_age(3600)
                    .finish(),
            )
            .service(index)
            .service(list_markets)
            .service(dex_profile)
            .service(dex_depth)
            .service(klines)
            .service(agg_trades)
            .service(freeze_balance)
            .service(dex_info)
            .service(list_orders)
            .service(recent_trades)
            .service(
                web::resource("/addMarket/{contract_address}")
                    .route(web::post().to(add_market)),
            )
            .service(echo)
        //.service(web::resource("/chemix/depth").route(web::get().to(depth)))
    })
        .bind(service.as_str())?
        .run()
        .await
}
