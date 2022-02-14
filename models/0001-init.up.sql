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
create unique index idx_local_chemix_tokens_address on chemix_tokens (address);

-- markets table
create table chemix_markets(
 id text primary key,
 base_token_address text ,
 base_token_symbol text ,
 quote_token_address text ,
 quote_token_symbol text ,
 matched_address text ,
 online  boolean ,
 up_at  timestamp ,
 down_at  timestamp ,
 updated_at timestamp ,
 created_at timestamp
);

-- trades table
create table chemix_trades(
  id text PRIMARY KEY,
  transaction_id integer , --admin处理的trade的序列号
  transaction_hash text,
  status text , --"matched","confirm"
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
create index idx_local_chemix_trades_taker on chemix_trades (taker);
create index idx_local_chemix_trades_maker on chemix_trades (maker);
create index idx_local_chemix_trades_taker_order_id  on chemix_trades (taker_order_id);
create index idx_local_chemix_trades_maker_order_id on chemix_trades (maker_order_id);
create index idx_local_chemix_trades_transaction_id on chemix_trades (transaction_id);
create index idx_local_chemix_trades_quotation  on chemix_trades (market_id, created_at);
create index idx_local_chemix_trades_delete on chemix_trades (status,transaction_hash,created_at);

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
create index idx_local_chemix_trades_tmp_quotation on chemix_trades_tmp (market_id,created_at);
create index idx_local_chemix_trades_tmp_recent on chemix_trades_tmp (market_id);
create index idx_local_chemix_trades_tmp_launch on chemix_trades_tmp (created_at,status);
create index idx_local_chemix_trades_tmp_status on chemix_trades_tmp (status);
create index idx_local_chemix_trades_tmp_txid on chemix_trades_tmp (transaction_id);
create index idx_local_chemix_trades_tmp_txhash on chemix_trades_tmp (transaction_hash,status);

-- orders table
create table chemix_orders(
  id text  primary key,
  market_id text ,
  account text ,
  side text ,
  price  numeric(32,8) ,
  amount  numeric(32,8) ,
  status text , --"full_filled","partial_filled","pending"
  available_amount  numeric(32,8) ,
  matched_amount  numeric(32,8) ,
  canceled_amount  numeric(32,8) ,
  confirmed_amount  numeric(32,8) ,
  updated_at  timestamp,
  created_at  timestamp
);
create index idx_local_chemix_myorders_status on chemix_orders (status);

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

create index  idx_local_chemix_orders_tmp_matche on chemix_orders_tmp (market_id, side, price, available_amount);
create index  idx_local_chemix_orders_tmp_orderbook on chemix_orders_tmp (market_id, available_amount, side);
create index  idx_local_chemix_orders_tmp_address on chemix_orders_tmp (trader_address,status);

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
-- create unique index idx_local_chemix_transactions_pendingTX on chemix_transactions (created_at,status,transaction_hash,id);
create unique index idx_local_chemix_transactions_pendingtx2 on chemix_transactions (created_at, status, transaction_hash);




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
create index idx_local_chemix_order_book_tmp_market_book on chemix_order_book_tmp (market_id,precision);
