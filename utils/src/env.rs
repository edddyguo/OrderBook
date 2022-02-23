use std::env;
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
    //业务模块具体处理是否必须从环境变量注入
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