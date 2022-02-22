use std::env;
/***
export CHEMIX_MODE=dev
export API_PORT=8010
export WS_PORT=8020
export PSQL=1
export REDIS=2
export CHAIN_RPC=2
export CHAIN_WS=2
export CHEMIX_MAIN=1
export CHEMIX_STORAGE
export CHEMIX_TOKEN_PROXY
export CHEMIX_VAULT
export CHEMIX_RELAYER_PRIKEY
export CHEMIX_BOT
*/
use std::collections::HashMap;
use std::ffi::OsString;
use std::string::String;
pub struct EnvConf{
    chemix_mode: Option<OsString>,
    api_port: Option<OsString>,
    ws_port: Option<OsString>,
    psql: Option<OsString>,
    redis: Option<OsString>,
    chain_rpc: Option<OsString>,
    chain_ws: Option<OsString>,
    chemix_main: Option<OsString>,
    chemix_storage: Option<OsString>,
    chemix_token_proxy: Option<OsString>,
    chemix_vault: Option<OsString>,
    chemix_relayer_prikey: Option<OsString>,
    chemix_bot_pri_key: Option<OsString>,
}

lazy_static! {
    static ref Conf: EnvConf = {
        EnvConf {
            chemix_mode: env::var_os("CHEMIX_MODE"),
            api_port: env::var_os("API_PORT"),
            ws_port: env::var_os("WS_PORT"),
            psql: env::var_os("PSQL"),
            redis: env::var_os("REDIS"),
            chain_rpc: env::var_os("CHAIN_RPC"),
            chain_ws: env::var_os("CHAIN_WS"),
            chemix_main: env::var_os("CHEMIX_MAIN"),
            chemix_storage: env::var_os("CHEMIX_STORAGE"),
            chemix_token_proxy: env::var_os("CHEMIX_TOKEN_PROXY"),
            chemix_vault: env::var_os("CHEMIX_VAULT"),
            chemix_relayer_prikey: env::var_os("CHEMIX_RELAYER_PRIKEY"),
            chemix_bot_pri_key: env::var_os("CHEMIX_BOT_PRIKEY"),
        }
    };
}