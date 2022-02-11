use crate::{Client, Clients};
use futures::{FutureExt, SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::from_str;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
//use warp::filters::ws::Message;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct TopicsRequest {
    topics: Vec<String>,
}

#[derive(Deserialize, Debug,PartialEq)]
pub enum WSMethod {
    #[serde(rename = "UNSUBSCRIBE")]
    UNSUBSCRIBE,
    #[serde(rename = "SUBSCRIBE")]
    SUBSCRIBE,
    #[serde(rename = "GET_PROPERTY")]
    GET_PROPERTY,
    #[serde(rename = "REGISTER")]
    REGISTER,
}
#[derive(Deserialize, Debug)]
pub struct TopicsRequest2 {
    method: WSMethod,
    params: Vec<String>,
}

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

    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e);
        }
    }));

    //client.sender = Some(client_sender);

    //第一次必须先注册
    let mut tmp1 = 0;
    let mut id = "".to_string();
    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("error receiving ws message : {}", e);
                break;
            }
        };

        let message = match msg.to_str() {
            Ok(v) => v,
            Err(_) => return,
        };

        let topics_req: TopicsRequest2 = match from_str(&message) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error while parsing message to topics request: {}", e);
                return;
            }
        };

        if tmp1 == 0 {
            if topics_req.method == WSMethod::REGISTER {
                let client = Client{
                    topics: vec![],
                    sender: Some(client_sender.clone())
                };
                //判断重复注册的情况
                id = topics_req.params[0].clone();
                clients.write().await.insert(topics_req.params[0].clone(), client);
                println!("{} connected", topics_req.params[0]);
                tmp1 = 1;
            }else {
                // must register before subscribe it
                client_sender.send(Ok(Message::text("error: must register before subscribe it")));
            }
        }else {
            client_msg(&id, msg, &clients).await;
        }

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

    if message == "ping" || message == "ping\n" {
        return;
    }

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
        println!("topics={:?}", topics_req.params);
        println!("topics={:?}", topics_req.method);
        //todo: match  method
        match topics_req.method {
            WSMethod::SUBSCRIBE => {
                v.topics = topics_req.params;
                if let Some(sender) = &v.sender {
                    //todo： 从psql或者redis或者合约拿到所有订单加工成depth的全量数据和aggtrade的最近50条数据
                    let _ = sender.send(Ok(Message::text("1111")));
                }
            }
            WSMethod::UNSUBSCRIBE => {
                for param in topics_req.params {
                    v.topics.retain(|x| x.to_string() != param);
                }
            }
            WSMethod::GET_PROPERTY => {
                todo!()
            }
            WSMethod::REGISTER => {
                //Have exist already,do nothing
            }
        }
    }
}
