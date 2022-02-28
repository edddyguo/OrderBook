use crate::{Client, Clients};
use futures::{FutureExt, SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{from_str, to_string};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
//use warp::filters::ws::Message;

use crate::handler::PublishRespond;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct TopicsRequest {
    topics: Vec<String>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum WSMethod {
    #[serde(rename = "UNSUBSCRIBE")]
    UNSUBSCRIBE,
    #[serde(rename = "SUBSCRIBE")]
    SUBSCRIBE,
    #[serde(rename = "GET_PROPERTY")]
    GET_PROPERTY,
    #[serde(rename = "PING")]
    PING,
    #[serde(rename = "PONG")]
    PONG,
}

/***{
"method": "SUBSCRIBE",
"params": {
"hash":"0x90a97d253608B2090326097a44eA289d172c30Ec",
"channel": "BTC-USDT@aggTrade",

}
}
***/

#[derive(Deserialize, Debug, PartialEq)]
pub struct PARAMS {
    hash: String,
    channel: Vec<String>,
}
#[derive(Deserialize, Debug)]
pub struct TopicsRequest2 {
    method: WSMethod,
    params: PARAMS,
}

//{"method": "PING","params":{"channel":[],"hash":""}}
//{"method": "SUBSCRIBE","params":{"channel":["BTC-USDT@kline_1d","BTC-USDT@aggTrade","BTC-USDT@depth"],"hash":""}}

//{"method": "SUBSCRIBE", "params": ["ethbusd@kline_1d","ethbusd@aggTrade","ethbusd@depth"]}
//["miniTicker@arr@3000ms", "ethbusd@aggTrade", "ethbusd@kline_1d", "ethbusd@depth"]
//{"method":"UNSUBSCRIBE","params":["!miniTicker@arr@3000ms","ethbusd@aggTrade","ethbusd@kline_1d","ethbusd@depth

pub async fn client_connection(
    ws: WebSocket,
    clients: Clients,
    //mut client: Client,
) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();
    let client_rcv = UnboundedReceiverStream::new(client_rcv); // <-- this

    //insert new client
    let id = Uuid::new_v4().simple().to_string();
    let client = Client {
        topics: vec![],
        user_address: None,
        sender: Some(client_sender.clone()),
    };

    let mut test1 = clients.write().await;
    test1.insert(id.clone(), client.clone());
    drop(test1);
    println!("1111---");

    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e);
        }
    }));

    //client.sender = Some(client_sender);

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("error receiving ws message : {}", e);
                break;
            }
        };
        println!("333--33");

        client_msg(&id, msg, &clients).await;
    }

    clients.write().await.remove(&id);
    println!("{} disconnected", id);
}

async fn client_msg(id: &str, msg: Message, clients: &Clients) {
    println!("received message from {}: {:?}", id, msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    //todo: 针对message做具体的订阅、取消订阅
    let topics_req: TopicsRequest2 = match from_str(&message) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error while parsing message to topics request: {}", e);
            return;
        }
    };

    let mut locked = clients.write().await;
    if let Some(v) = locked.get_mut(id) {
        println!("topics={:?}", topics_req.params.channel);
        println!("topics={:?}", topics_req.method);
        //todo: match  method
        match topics_req.method {
            WSMethod::SUBSCRIBE => {
                v.topics = topics_req.params.channel;
                match topics_req.params.hash.as_str() {
                    "" => {
                        v.user_address = None;

                    },
                    _ =>{
                        v.user_address = Some(topics_req.params.hash.to_lowercase());
                    }
                }
                println!("v----topics={:?}", v.topics);
                if let Some(_sender) = &v.sender {
                    //todo: 可能要推全量数据
                }
            }
            WSMethod::UNSUBSCRIBE => {
                for param in topics_req.params.channel {
                    v.topics.retain(|x| x.to_string() != param);
                }
            }
            WSMethod::GET_PROPERTY => {
                todo!()
            }
            WSMethod::PING => {
                let respond = PublishRespond {
                    method: "PONG".to_string(),
                    channel: "".to_string(),
                    data: "".to_string(),
                };
                let respond_str = serde_json::to_string(&respond).unwrap();
                if let Some(sender) = &v.sender {
                    let _ = sender.send(Ok(Message::text(&respond_str)));
                }
            }
            WSMethod::PONG => {}
        }
    }
}
