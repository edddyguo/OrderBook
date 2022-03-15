mod depth;
mod error_code;
mod kline;
mod market;
mod order;
mod trade;

use actix_cors::Cors;
use actix_web::{error, get, post, web, App, HttpResponse, HttpServer, Responder};
use log::info;
use std::env;

use chemix_models::market::{get_markets, list_markets as list_markets2};
use chemix_models::order::{get_order_volume, list_orders as list_orders2, EngineOrderTmp2, OrderFilter, get_user_number};
use chemix_models::snapshot::get_snapshot;
use chemix_models::trade::{list_trades, TradeFilter};
use chemix_models::TimeScope;
use common::utils::time::{get_current_time, get_unix_time, time2unix};
use serde::{Deserialize, Serialize};
use chemix_models::thaws::{list_thaws3, ThawsFilter};

use chemix_models::tokens::get_token;

use common::utils::math::u256_to_f64;

use crate::order::{get_order_detail, OrderDetail};
use common::types::order::Side as OrderSide;
use common::types::order::Status as OrderStatus;
use common::types::trade::Status as TradeStatus;

#[macro_use]
extern crate common;

#[get("/{id}/{name}/index.html")]
async fn index(web::Path((id, name)): web::Path<(u32, String)>) -> impl Responder {
    format!("Hello {}! id:{}", name, id)
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
    snapshot_time: u64,
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
    let engine = common::env::CONF.chemix_main.to_owned().unwrap();
    let vault = common::env::CONF.chemix_vault.to_owned().unwrap();
    let proxy = common::env::CONF.chemix_token_proxy.to_owned().unwrap();

    let dex_info = DexInfo {
        engine_address: engine.into_string().unwrap(),
        vault_address: vault.into_string().unwrap(),
        proxy_address: proxy.into_string().unwrap(),
    };
    respond_json(
        200,
        "".to_string(),
        serde_json::to_string(&dex_info).unwrap(),
    )
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

#[derive(Serialize, Debug, Default)]
pub struct MarketInfoTmp1 {
    pub id: String,
    pub base_token_address: String,
    base_token_symbol: String,
    pub base_contract_decimal: u32,
    base_front_decimal: u32,
    pub quote_token_address: String,
    quote_token_symbol: String,
    pub quote_contract_decimal: u32,
    quote_front_decimal: u32,
    seven_day_volume: f64,
    twenty_four_hour_volume: f64,
    cvt_url: String,
    show_cvt: bool,
    snapshot_time: String,
}

#[get("/chemix/listMarkets")]
async fn list_markets(web::Path(()): web::Path<()>) -> impl Responder {
    let mut markets = Vec::<MarketInfoTmp1>::new();
    let db_markets = list_markets2();
    let now = get_current_time();
    for db_market in db_markets {
        let seven_day_volume = get_order_volume(TimeScope::SevenDay, &db_market.id);
        let twenty_four_hour_volume = get_order_volume(TimeScope::OneDay, &db_market.id);
        let token = get_token(db_market.base_token_symbol.as_str()).unwrap();
        let data = MarketInfoTmp1 {
            id: db_market.id,
            base_token_address: db_market.base_token_address,
            base_token_symbol: db_market.base_token_symbol,
            base_contract_decimal: db_market.base_contract_decimal,
            base_front_decimal: db_market.base_front_decimal,
            quote_token_address: db_market.quote_token_address,
            quote_token_symbol: db_market.quote_token_symbol,
            quote_contract_decimal: db_market.quote_contract_decimal,
            quote_front_decimal: db_market.quote_front_decimal,
            seven_day_volume: u256_to_f64(seven_day_volume, 15),
            twenty_four_hour_volume: u256_to_f64(twenty_four_hour_volume, 15),
            cvt_url: token.cvt_url,
            show_cvt: token.show_cvt,
            snapshot_time: now.clone(),
        };
        markets.push(data);
    }
    respond_json(
        200,
        "".to_string(),
        serde_json::to_string(&markets).unwrap(),
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
#[derive(Deserialize, Serialize)]
struct DepthRequest {
    market_id: String,
    limit: u32,
}

#[get("/chemix/depth")]
async fn dex_depth(web::Query(info): web::Query<DepthRequest>) -> String {
    format!("symbol222 {}, limit:{}", info.market_id, info.limit);
    let market_info = get_markets(info.market_id.as_str()).unwrap();
    let base_decimal = market_info.base_contract_decimal;
    let quote_decimal = market_info.quote_contract_decimal;
    //todo:BtreeMap
    let mut available_buy_orders = list_orders2(OrderFilter::AvailableOrders(
        info.market_id.clone(),
        OrderSide::Buy,
    ))
    .unwrap();
    let mut available_sell_orders = list_orders2(OrderFilter::AvailableOrders(
        info.market_id.clone(),
        OrderSide::Sell,
    ))
    .unwrap();

    available_buy_orders.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap().reverse());
    available_sell_orders.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());

    info!("0001__{:?}", available_buy_orders);
    info!("0002__{:?}", available_sell_orders);
    let mut asks = Vec::<(f64, f64)>::new();
    let mut bids = Vec::<(f64, f64)>::new();

    'buy_orders: for available_buy_order in available_buy_orders {
        'bids: for mut bid in bids.iter_mut() {
            if u256_to_f64(available_buy_order.price, quote_decimal) == bid.0 {
                bid.1 += u256_to_f64(available_buy_order.available_amount, base_decimal);
                continue 'buy_orders;
            }
        }
        if bids.len() as u32 == info.limit {
            break 'buy_orders;
        }
        bids.push((
            u256_to_f64(available_buy_order.price, quote_decimal),
            u256_to_f64(available_buy_order.available_amount, base_decimal),
        ));
    }

    'sell_orders: for available_sell_order in available_sell_orders {
        'asks: for mut ask in asks.iter_mut() {
            if u256_to_f64(available_sell_order.price, quote_decimal) == ask.0 {
                ask.1 += u256_to_f64(available_sell_order.available_amount, base_decimal);
                continue 'sell_orders;
            }
        }
        if asks.len() as u32 == info.limit {
            break 'sell_orders;
        }
        asks.push((
            u256_to_f64(available_sell_order.price, quote_decimal),
            u256_to_f64(available_sell_order.available_amount, base_decimal),
        ));
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
    market_id: String,
    limit: u32,
}

#[get("/chemix/aggTrades")]
async fn agg_trades(web::Query(info): web::Query<AggTradesRequest>) -> impl Responder {
    let market = get_markets(&info.market_id).unwrap();
    let (base_decimal, quote_decimal) =
        (market.base_contract_decimal, market.quote_contract_decimal);
    let trades = list_trades(TradeFilter::MarketId(info.market_id.clone(),info.limit))
    .iter()
    .map(|x| trade::Trade {
        id: x.id.clone(),
        transaction_hash: x.transaction_hash.clone(),
        market_id: info.market_id.clone(),
        price: u256_to_f64(x.price, quote_decimal),
        amount: u256_to_f64(x.amount, base_decimal),
        height: x.block_height,
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
    market_id: String,
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
    let cec_token_decimal = 15u32;
    //todo: 还未生成快照的时间点
    //current_and_yesterday_sanpshot
    let cays = get_snapshot().unwrap();
    let price = cays.0.cec_price;
    let currentTVL = cays.0.order_volume - cays.0.withdraw;
    let cumulativeTransactions = cays.0.transactions as u32;
    let cumulativeTraders = cays.0.traders as u32;
    let tradingPairs = cays.0.trading_pairs as u8;
    let current_transcations = cays.0.transactions as u32;
    let snapshot_time = cays.0.snapshot_time as u64;
    let current_trade_volume = cays.0.trade_volume;

    let yesterday_traders = cays.1.traders as u32;
    let yesterday_trader_volume = cays.1.trade_volume;
    let yesterday_transcations = cays.1.transactions as u32;
    let yesterdayTVL = cays.1.order_volume - cays.1.withdraw;

    let profile = DexProfile {
        cumulativeTVL: u256_to_f64(currentTVL, cec_token_decimal),
        cumulativeTransactions,
        cumulativeTraders,
        numberOfTraders: get_user_number(TimeScope::OneDay),
        tradingVolume: u256_to_f64(
            current_trade_volume - yesterday_trader_volume,
            cec_token_decimal,
        ),
        numberOfTransactions: current_transcations - yesterday_transcations,
        TVL: u256_to_f64(currentTVL - yesterdayTVL, cec_token_decimal),
        tradingPairs,
        price: u256_to_f64(price, cec_token_decimal),
        snapshot_time,
    };
    respond_json(
        200,
        "".to_string(),
        serde_json::to_string(&profile).unwrap(),
    )
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
    market_id: String,
    account: String,
    limit: u32,
}

#[get("/chemix/listOrders")]
async fn list_orders(web::Query(info): web::Query<ListOrdersRequest>) -> impl Responder {
    let market_info = get_markets(info.market_id.as_str()).unwrap();
    let (base_decimal,quote_decimal) = (market_info.base_contract_decimal,market_info.quote_contract_decimal);
    let account = info.account.clone().to_lowercase();
    let market_id = info.market_id.clone();
    let orders = list_orders2(OrderFilter::UserOrders(
        market_id.clone(),
        account.clone(),
        OrderStatus::Pending,
        OrderStatus::PartialFilled,
        info.limit,
    ))
    .unwrap();

    let mut orders = orders
        .iter()
        .map(|x| EngineOrderTmp2 {
            id: info.market_id.clone(),
            transaction_hash: x.transaction_hash.clone(),
            thaws_hash: "".to_string(),
            index: x.index.to_string(),
            account: x.account.clone(),
            price: u256_to_f64(x.price, quote_decimal),
            amount: u256_to_f64(x.amount, base_decimal),
            matched_amount: u256_to_f64(x.matched_amount, base_decimal),
            side: x.side.clone(),
            status: x.status.as_str().to_string(),
            created_at: time2unix(x.created_at.clone()),
        })
        .collect::<Vec<EngineOrderTmp2>>();
    let thaws = list_thaws3(ThawsFilter::NotConfirmed(market_id.clone(),account.clone()));
    //todo：优化
    let mut mock_order = thaws.iter().map(|x| {
        //todo: thaws 的数据结构
        let account = format!("{:?}", x.account);
        let origin_order = list_orders2(OrderFilter::ById(x.order_id.clone())).unwrap();
        EngineOrderTmp2 {
            id: x.market_id.clone(),
            transaction_hash: origin_order[0].transaction_hash.clone(),
            thaws_hash: x.thaws_hash.clone(),
            index: origin_order[0].index.to_string(),
            account,
            price: u256_to_f64(x.price, quote_decimal),
            amount: u256_to_f64(x.amount, base_decimal),
            matched_amount: u256_to_f64(x.amount, base_decimal),//tmpcode
            side: x.side.clone(),
            status: "thawing".to_string(), //tmp code
            created_at: time2unix(origin_order[0].created_at.clone()),
        }
    }).collect::<Vec<EngineOrderTmp2>>();
    orders.append(&mut mock_order);
    orders.sort_by(|a,b| {
        b.created_at.partial_cmp(&a.created_at).unwrap()
    });

    respond_json(200, "".to_string(), serde_json::to_string(&orders).unwrap())
}

//todo 文档同步
#[derive(Deserialize, Serialize, Debug)]
struct OrderHistoryRequest {
    market_id: String,
    account: String,
    limit: u32,
}

#[get("/chemix/orderHistory")]
async fn order_history(web::Query(info): web::Query<OrderHistoryRequest>) -> impl Responder {
    let account = info.account.clone().to_lowercase();
    let market_id = info.market_id.clone();
    let orders = list_orders2(OrderFilter::UserOrders(
        market_id,
        account,
        OrderStatus::FullFilled,
        OrderStatus::Canceled,
        info.limit,
    ))
    .unwrap();
    let orders = orders
        .iter()
        .map(|x| get_order_detail(x))
        .collect::<Vec<OrderDetail>>();
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
    market_id: String,
    limit: u32,
}

#[get("/chemix/recentTrades")]
async fn recent_trades(web::Query(info): web::Query<RecentTradesRequest>) -> impl Responder {
    let market_info = get_markets(info.market_id.as_str()).unwrap();
    let (base_decimal,quote_decimal) = (market_info.base_contract_decimal,market_info.quote_contract_decimal);
    let account = info.account.clone().to_lowercase();
    let trades = list_trades(TradeFilter::Recent(account.clone(),info.market_id.clone(),
                                                 TradeStatus::Confirmed,info.limit));
    let trades = trades
        .iter()
        .map(|x| {
            let side = if account == x.taker {
                x.taker_side.clone()
            } else {
                x.taker_side.contrary()
            };
            trade::Trade {
                id: x.id.clone(),
                transaction_hash: x.transaction_hash.clone(),
                market_id: info.market_id.clone(),
                price: u256_to_f64(x.price, quote_decimal),
                amount: u256_to_f64(x.amount, base_decimal),
                height: 12345i32,
                // fixme: maybe side?
                taker_side: side,
                updated_at: time2unix(x.created_at.clone()),
            }
        })
        .collect::<Vec<trade::Trade>>();
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
        App::new()
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
            .service(order_history)
            .service(
                web::resource("/addMarket/{contract_address}")
                    .route(web::post().to(add_market)),
            )
            .service(echo)
    })
    .bind(service.as_str())?
    .run()
    .await
}
