-- tokens table
create table chemix_tokens(
 symbol text primary key,
 name text,
 address text ,
 front_decimals integer,
 base_contract_decimal integer,
 cvt_url text,
 show_cvt boolean,
 updated_at timestamp,
 created_at timestamp
);
create unique index idx_chemix_tokens_symbol on chemix_tokens (symbol);

-- markets table
create table chemix_markets(
 id text primary key,
 base_token_address text ,
 base_token_symbol text ,
 base_front_decimal integer,
 base_contract_decimal integer,
 quote_token_address text ,
 quote_token_symbol text ,
 quote_front_decimal integer,
 quote_contract_decimal integer,
 online  boolean ,
 up_at  timestamp ,
 down_at  timestamp ,
 updated_at timestamp ,
 created_at timestamp
);
create unique index idx_chemix_markets_symbol on chemix_markets (online);


-- trades table
create table chemix_trades(
  id text PRIMARY KEY,
  block_height integer , --admin处理的trade的序列号
  transaction_hash text,
  hash_data  text ,
  status text , --"matched","confirm"
  market_id text ,
  maker  text ,
  taker  text ,
  price text ,
  amount text ,
  taker_side text ,
  maker_order_id  text ,
  taker_order_id text ,
  updated_at timestamp ,
  created_at timestamp
);

create index idx_chemix_trades_market on chemix_trades (market_id);
create index idx_chemix_trades_recent on chemix_trades (taker,maker,status);
create index idx_chemix_trades_confirm  on chemix_trades (hash_data,block_height);
create index idx_chemix_trades_status on chemix_trades (status);

-- orders table
create table chemix_orders(
  id text  primary key,
  index integer,
  transaction_hash text,
  block_height integer,
  hash_data text,
  market_id text ,
  account text ,
  side text ,
  price  text ,
  amount  text ,
  status text , --"full_filled","partial_filled","pending"
  available_amount  text ,
  matched_amount  text ,
  canceled_amount  text ,
  updated_at  timestamp,
  created_at  timestamp
);

create index idx_chemix_orders_index on chemix_orders (index);
create index idx_chemix_orders_available on chemix_orders (market_id,status);
create index idx_chemix_orders_users on chemix_orders (account,market_id,status,status);

create table chemix_thaws(
  order_id text  primary key,
  account text,
  market_id text ,
  transaction_hash text ,
  block_height integer ,
  thaws_hash  text ,
  side  text ,
  status  text ,
  amount  text ,
  price  text ,
  updated_at  timestamp,
  created_at  timestamp
);

create index idx_chemix_thaws_uncertain on chemix_thaws (market_id,account,status);
create index idx_chemix_thaws_status on chemix_thaws (status);
create index idx_chemix_thaws_delay_confirm on chemix_thaws (thaws_hash,block_height,status);


create table chemix_snapshot(
  traders int default 0,
  transactions  int default 0,
  order_volume text default '',
  withdraw text default '',
  trade_volume text default '',
  trading_pairs int default 0,
  cec_price text default '',
  snapshot_time bigint,
  updated_at  timestamp default now(),
  created_at  timestamp default now()
);