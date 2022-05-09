#![allow(missing_docs)]
use std::env;

use std::ffi::OsString;

///read config data for env
pub struct EnvConf {
    /// dev or pro
    pub chemix_mode: Option<OsString>,
    ///http service port
    pub api_port: Option<OsString>,
    /// ws servie prot
    pub ws_port: Option<OsString>,
    /// psql connect url
    pub psql: Option<OsString>,
    /// redis
    pub redis: Option<OsString>,
    /// eth rpc url
    pub chain_rpc: Option<OsString>,
    /// chain id
    pub chain_id: Option<OsString>,
    /// chain ws url
    pub chain_ws: Option<OsString>,
    /// chemix main address
    pub chemix_main: Option<OsString>,
    /// chemix stroage contract address
    pub chemix_storage: Option<OsString>,
    /// chemix token proxy contract address
    pub chemix_token_proxy: Option<OsString>,
    /// chemix vault contract address
    pub chemix_vault: Option<OsString>,
    ///pri key for settlement
    pub chemix_relayer_prikey: Option<OsString>,
    ///bot key
    pub chemix_bot_prikey: Option<OsString>,
}

lazy_static! {
    //业务模块具体处理是否必须从环境变量注入
    pub static ref CONF: EnvConf = {
        EnvConf {
            chemix_mode: env::var_os("CHEMIX_MODE"),
            api_port: env::var_os("API_PORT"),
            ws_port: env::var_os("WS_PORT"),
            psql: env::var_os("PSQL"),
            redis: env::var_os("REDIS"),
            chain_rpc: env::var_os("CHAIN_RPC"),
            chain_id: env::var_os("CHAIN_ID"),
            chain_ws: env::var_os("CHAIN_WS"),
            chemix_main: env::var_os("CHEMIX_MAIN"),
            chemix_storage: env::var_os("CHEMIX_STORAGE"),
            chemix_token_proxy: env::var_os("CHEMIX_TOKEN_PROXY"),
            chemix_vault: env::var_os("CHEMIX_VAULT"),
            chemix_relayer_prikey: env::var_os("CHEMIX_RELAYER_PRIKEY"),
            chemix_bot_prikey: env::var_os("CHEMIX_BOT_PRIKEY"),
        }
    };
}
