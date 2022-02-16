use serde::{Serialize};

#[derive(Serialize)]
struct Trade {
    code: u8,
    msg: String, //200 default success
    data: String,
}
//*               [50000.0,10.0001],
