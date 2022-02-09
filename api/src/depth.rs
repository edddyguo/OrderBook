use serde::{Serialize,Deserialize};

#[derive(Serialize)]
struct Trade {
    code : u8,
    msg : String,   //200 default success
    data : String,
}