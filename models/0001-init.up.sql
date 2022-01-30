-- tokens table
create table chemix_tokens(
 symbol text primary key,
 name text,
 address text ,
 decimals integer ,
 bsc_assetID text ,
 bsc_address text ,
 created_at timestamp
);
create unique index idx_chemix_tokens_address on chemix_tokens (address);

-- markets table
create table chemix_markets(
 id text primary key,
 base_token_address text ,
 base_token_symbol text ,
 quote_token_address text ,
 quote_token_symbol text ,
 online  boolean ,
 up_at  timestamp ,
 down_at  timestamp ,
 updated_at timestamp ,
 created_at timestamp
);

-- trades table
create table chemix_trades(
  id text PRIMARY KEY,
  transaction_id integer ,
  transaction_hash text,
  status text ,
  market_id text ,
  maker  text ,
  taker  text ,
  price numeric(32,8) ,
  amount numeric(32,8) ,
  taker_side text ,
  maker_order_id  text ,
  taker_order_id text ,
  updated_at timestamp ,
  created_at timestamp
);
create index idx_chemix_trades_taker on chemix_trades (taker);
create index idx_chemix_trades_maker on chemix_trades (maker);
create index idx_chemix_trades_taker_order_id  on chemix_trades (taker_order_id);
create index idx_chemix_trades_maker_order_id on chemix_trades (maker_order_id);
create index idx_chemix_trades_transaction_id on chemix_trades (transaction_id);
create index idx_chemix_trades_quotation  on chemix_trades (market_id, created_at);
create index idx_chemix_trades_delete on chemix_trades (status,transaction_hash,created_at);

create table chemix_trades_tmp(
  id text PRIMARY KEY,
  transaction_id integer ,
  transaction_hash text,
  status text ,
  market_id text ,
  maker  text ,
  taker  text ,
  price numeric(32,8) ,
  amount numeric(32,8) ,
  taker_side text ,
  maker_order_id  text ,
  taker_order_id text ,
  updated_at timestamp ,
  created_at timestamp
);
create index idx_chemix_trades_tmp_quotation on chemix_trades_tmp (market_id,created_at);
create index idx_chemix_trades_tmp_recent on chemix_trades_tmp (market_id);
create index idx_chemix_trades_tmp_launch on chemix_trades_tmp (created_at,status);
create index idx_chemix_trades_tmp_status on chemix_trades_tmp (status);
create index idx_chemix_trades_tmp_txid on chemix_trades_tmp (transaction_id);
create index idx_chemix_trades_tmp_txhash on chemix_trades_tmp (transaction_hash,status);

-- orders table
create table chemix_orders(
  id text  primary key,
  trader_address text ,
  market_id text ,
  side text ,
  price  numeric(32,8) ,
  amount  numeric(32,8) ,
  status text ,
  type text ,
  available_amount  numeric(32,8) ,
  confirmed_amount  numeric(32,8) ,
  canceled_amount  numeric(32,8) ,
  pending_amount  numeric(32,8) ,
  updated_at  timestamp,
  created_at  timestamp,
  signature text ,
  expire_at  bigint
);
create index idx_chemix_myorders_status on chemix_orders (status);
create index idx_chemix_myorders_address on chemix_orders (trader_address);
create index idx_chemix_myorders_v1 on chemix_orders (trader_address, market_id,side,created_at);
create index idx_chemix_myorders_v2 on chemix_orders (trader_address, market_id,created_at);
create index idx_chemix_myorders_v3 on chemix_orders (trader_address,side,created_at);


create table chemix_orders_tmp(
  id text  primary key,
  trader_address text ,
  market_id text ,
  side text ,
  price  numeric(32,8) ,
  amount  numeric(32,8) ,
  status text ,
  type text ,
  available_amount  numeric(32,8) ,
  confirmed_amount  numeric(32,8) ,
  canceled_amount  numeric(32,8) ,
  pending_amount  numeric(32,8) ,
  updated_at  timestamp,
  created_at  timestamp,
  signature text ,
  expire_at  bigint
);

create index  idx_chemix_orders_tmp_matche on chemix_orders_tmp (market_id, side, price, available_amount);
create index  idx_chemix_orders_tmp_orderbook on chemix_orders_tmp (market_id, available_amount, side);
create index  idx_chemix_orders_tmp_address on chemix_orders_tmp (trader_address,status);

-- transactions table
create table chemix_transactions(
  id SERIAL PRIMARY KEY,
  transaction_hash text,
  market_id text ,
  status text ,
  contract_status text ,
  updated_at  timestamp,
  created_at timestamp
);
--Update index
-- create unique index idx_chemix_transactions_pendingTX on chemix_transactions (created_at,status,transaction_hash,id);
create unique index idx_chemix_transactions_pendingtx2 on chemix_transactions (created_at, status, transaction_hash);


create table chemix_bridge(
  id text PRIMARY KEY,
  address  text default '',
  token_name text default '',
  amount numeric(32,8) default 0,
  side  text default '', 
  master_txid text default '',
  master_txid_status text default '',
  child_txid  text default '',
  child_txid_status  text default '',
  fee_asset  text default '', 
  fee_amount  text default '',
  updated_at  timestamp default now(),
  created_at  timestamp default now()
);

create index idx_chemix_bridge_pending_decode on chemix_bridge (address,master_txid_status,created_at);
create index idx_chemix_bridge_my_bridge on chemix_bridge (address,created_at);
create index idx_chemix_bridge_pending_trade on chemix_bridge (side,master_txid_status,child_txid_status,created_at);

create index idx_chemix_bridge_statistical on chemix_bridge (side,token_name);




create table chemix_market_quotation_tmp(
  market_id text PRIMARY KEY,
  price  numeric(32,8) default 0,
  ratio  numeric(32,8) default 0,
  volume numeric(32,8) default 0,
  CNYC_price numeric(32,8) default 0,
  maxprice numeric(32,8) default 0,
  minprice numeric(32,8) default 0,
  min_CNYC_price numeric(32,8) default 0,
  max_CNYC_price numeric(32,8) default 0,
  symbol  text default '',
  updated_at  timestamp default now(),
  created_at  timestamp default now()
);

create table chemix_order_book_tmp(
  market_id text default 0,
  precision  int default 0,
  order_book text default '',
  updated_at  timestamp default now(),
  created_at  timestamp default now()
);
create index idx_chemix_order_book_tmp_market_book on chemix_order_book_tmp (market_id,precision);

/**
id                   | text                        |          | not null |
 address              | text                        |          |          |
 deposit_assetid      | text                        |          |          |
 deposit_amount       | numeric(32,18)              |          |          |
 deposit_token_name   | text                        |          |          |
 deposit_price        | numeric(32,18)              |          |          |
 interest_rate        | numeric(32,18)              |          |          |
 cdp_id               | integer                     |          |          |
 status               | text                        |          |          |
 zhiya_rate           | numeric(32,18)              |          |          |
 usage                | text                        |          |          |
 borrow_amount        | numeric(32,18)              |          |          |
 borrow_time          | integer                     |          |          |
 repaid_amount        | numeric(32,18)              |          |          |
 should_repaid_amount | numeric(32,18)              |          |          |
 cdp_address          | text                        |          |          |
 updated_at           | timestamp without time zone |          |          |
 created_at           | timestamp without time zone |          |          |
索引：****/
create table chemix_borrows(
  id text default '',
  address text default '',
  token text default '',
  trade_id text default '',
  amount  numeric(32,18) default 0,
  repaid_amount numeric(32,18) default 0,
  rate numeric(32,18) default 0, --一小时利率
  status text default '', --open,close,forced 爆仓
  updated_at  timestamp default now(),
  created_at  timestamp default now()
);
