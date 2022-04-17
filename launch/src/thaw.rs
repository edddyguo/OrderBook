use crate::U256;
use crate::{OrderSide, ThawStatus};
use chemix_chain::chemix::vault::{ThawBalances, Vault};
use chemix_chain::chemix::{ChemixContractClient, ThawBalances2};
use chemix_chain::{gen_txid, send_raw_transaction};
use chemix_models::market::get_markets;
use chemix_models::thaws::{list_thaws, update_thaws, ThawsFilter, ThawsPO, UpdateThaw};
use common::queue::QueueType;
use common::utils::algorithm::{sha256, u8_arr_from_str, u8_arr_to_str};
use common::utils::math::u256_to_f64;
use common::utils::time::get_current_time;
use ethers::types::Address;
use rsmq_async::{Rsmq, RsmqConnection};
use std::str::FromStr;
use std::sync::{Arc, RwLock};

pub async fn send_launch_thaw(
    vault_settel_client: Arc<RwLock<ChemixContractClient<Vault>>>,
    pending_thaws: Vec<ThawsPO>,
) {
    let mut thaw_infos = Vec::new();
    for pending_thaw in pending_thaws.clone() {
        let market = get_markets(pending_thaw.market_id.as_str()).unwrap();
        let token_base_decimal = teen_power!(market.base_contract_decimal);
        let (token_address, amount, decimal) = match pending_thaw.side {
            OrderSide::Sell => {
                info!("available_amount {}", pending_thaw.amount);
                (
                    market.base_token_address,
                    pending_thaw.amount,
                    market.base_contract_decimal,
                )
            }
            OrderSide::Buy => {
                info!(
                    "available_amount {},price {},thaw_amount {}",
                    pending_thaw.amount,
                    pending_thaw.price,
                    pending_thaw.amount * pending_thaw.price / token_base_decimal
                );
                (
                    market.quote_token_address,
                    pending_thaw.amount * pending_thaw.price / token_base_decimal,
                    market.quote_contract_decimal,
                )
            }
        };
        thaw_infos.push(ThawBalances {
            token: Address::from_str(token_address.as_str()).unwrap(),
            from: Address::from_str(pending_thaw.account.as_str()).unwrap(),
            decimal: decimal as u32,
            amount,
        });
    }
    info!("start thaw balance,all thaw info {:?}", thaw_infos);

    let order_json = format!(
        "{}{}",
        serde_json::to_string(&thaw_infos).unwrap(),
        get_current_time()
    );

    let cancel_id = u8_arr_from_str(sha256(order_json));

    let raw_data = vault_settel_client
        .read()
        .unwrap()
        .thaw_balances(thaw_infos.clone(), cancel_id)
        .await;
    let txid = gen_txid(&raw_data);

    let cancel_id_str = u8_arr_to_str(cancel_id);
    let now = get_current_time();
    let mut pending_thaws2 = pending_thaws
        .iter()
        .map(|x| UpdateThaw {
            order_id: x.order_id.clone(),
            cancel_id: cancel_id_str.to_string(),
            block_height: 0,
            transaction_hash: txid.clone(),
            status: ThawStatus::Launched,
            updated_at: &now,
        })
        .collect::<Vec<UpdateThaw>>();
    update_thaws(&pending_thaws2);
    //todo: 此时节点问题或者分叉,或者gas不足
    let receipt = send_raw_transaction(raw_data).await;
    info!("finish thaw balance res:{:?}", receipt);
    let transaction_hash = format!("{:?}", receipt.transaction_hash);
    assert_eq!(txid, transaction_hash);
    let height = receipt.block_number.unwrap().as_u32();
    for pending_thaw in pending_thaws2.iter_mut() {
        pending_thaw.block_height = height;
    }
    update_thaws(&pending_thaws2);
    //todo: 取消的订单也超过100这种
}

pub async fn deal_launched_thaws(
    new_thaw_flags: Vec<String>,
    arc_queue: Arc<RwLock<Rsmq>>,
    height: u32,
) {
    let now = get_current_time();
    for new_thaw_flag in new_thaw_flags {
        //如果已经确认的跳过，可能发生在系统重启的时候
        let pending_thaws = list_thaws(ThawsFilter::DelayConfirm(&new_thaw_flag, height));
        if pending_thaws.is_empty() {
            warn!(
                "This thaw hash {} have already dealed,and jump it",
                new_thaw_flag.clone()
            );
            continue;
        }
        let iters = pending_thaws.group_by(|a, b| a.market_id == b.market_id);

        //所有交易对的解冻一起更新
        let mut update_thaws_arr = Vec::new();
        for iter in iters {
            let mut thaw_infos = Vec::new();
            for pending_thaw in iter.to_vec() {
                update_thaws_arr.push(UpdateThaw {
                    order_id: pending_thaw.order_id,
                    cancel_id: pending_thaw.thaws_hash.clone(),
                    block_height: height,
                    transaction_hash: pending_thaw.transaction_hash.clone(),
                    status: ThawStatus::Confirmed,
                    updated_at: &now,
                });

                let market = get_markets(pending_thaw.market_id.as_str()).unwrap();
                let token_base_decimal = teen_power!(market.base_contract_decimal);
                let (token_address, amount, decimal) = match pending_thaw.side {
                    OrderSide::Sell => {
                        info!("available_amount {}", pending_thaw.amount);
                        (
                            market.base_token_address,
                            pending_thaw.amount,
                            market.base_contract_decimal,
                        )
                    }
                    OrderSide::Buy => {
                        info!(
                            "available_amount {},price {},thaw_amount {}",
                            pending_thaw.amount,
                            pending_thaw.price,
                            pending_thaw.amount * pending_thaw.price / token_base_decimal
                        );
                        (
                            market.quote_token_address,
                            pending_thaw.amount * pending_thaw.price / token_base_decimal,
                            market.quote_contract_decimal,
                        )
                    }
                };
                thaw_infos.push(ThawBalances {
                    token: Address::from_str(token_address.as_str()).unwrap(),
                    from: Address::from_str(pending_thaw.account.as_str()).unwrap(),
                    decimal: decimal as u32,
                    amount,
                });
            }

            let thaw_infos2 = thaw_infos
                .iter()
                .map(|x| ThawBalances2 {
                    token: x.token,
                    from: x.from,
                    amount: u256_to_f64(x.amount, x.decimal),
                })
                .collect::<Vec<ThawBalances2>>();
            let json_str = serde_json::to_string(&thaw_infos2).unwrap();

            arc_queue
                .write()
                .unwrap()
                .send_message(QueueType::Thaws.to_string().as_str(), json_str, None)
                .await
                .expect("failed to send message");
        }
        update_thaws(&update_thaws_arr);
    }
}
