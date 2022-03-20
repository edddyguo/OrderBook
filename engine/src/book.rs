use std::cmp::Ordering;
use std::collections::BTreeMap;

use chemix_models::order::OrderInfo;
use common::types::order::Side as OrderSide;
use ethers_core::types::U256;
use serde::Serialize;

//buy 倒叙
#[derive(Clone, Serialize, Debug, Eq)]
pub struct BuyPriority {
    pub price: U256,
    pub order_index: u32,
}

impl PartialEq<Self> for BuyPriority {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.order_index == other.order_index
    }
}

impl PartialOrd for BuyPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BuyPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        let price_cmp_res = self.price.partial_cmp(&other.price);
        let created_cmp_res = self.order_index.partial_cmp(&other.order_index);
        if price_cmp_res == Some(Ordering::Greater) {
            Ordering::Less
        } else if price_cmp_res == Some(Ordering::Equal)
            && created_cmp_res == Some(Ordering::Less)
        {
            Ordering::Greater
        } else {
            Ordering::Greater
        }
    }
}

//sell正序
#[derive(Clone, Serialize, Debug, Eq)]
pub struct SellPriority {
    pub price: U256,
    pub order_index: u32,
}

impl PartialEq<Self> for SellPriority {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.order_index == other.order_index
    }
}

impl PartialOrd<Self> for SellPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SellPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        let price_cmp_res = self.price.partial_cmp(&other.price);
        let created_cmp_res = self.order_index.partial_cmp(&other.order_index);
        if price_cmp_res == Some(Ordering::Greater) {
            Ordering::Greater
            //todo: 测试结果导向，但是逻辑没理解
        } else if price_cmp_res == Some(Ordering::Equal)
            && created_cmp_res == Some(Ordering::Greater)
        {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

#[derive(Clone, Serialize, Debug)]
pub struct BookValue {
    pub id: String,
    pub account: String,
    pub side: OrderSide,
    pub amount: U256,
}

#[derive(Clone, Serialize, Debug)]
pub struct Book {
    pub buy: BTreeMap<BuyPriority, BookValue>,
    pub sell: BTreeMap<SellPriority, BookValue>,
}

type EngineBuyOrder = (BuyPriority, BookValue);
type EngineSellOrder = (SellPriority, BookValue);

pub fn gen_engine_buy_order(order: &OrderInfo) -> EngineBuyOrder {
    (
        BuyPriority {
            price: order.price,
            order_index: order.index,
        },
        BookValue {
            id: order.id.clone(),
            account: order.account.clone(),
            side: order.side.clone(),
            amount: order.available_amount,
        },
    )
}

pub fn gen_engine_sell_order(order: &OrderInfo) -> EngineSellOrder {
    (
        SellPriority {
            price: order.price,
            order_index: order.index,
        },
        BookValue {
            id: order.id.clone(),
            account: order.account.clone(),
            side: order.side.clone(),
            amount: order.available_amount,
        },
    )
}
