use std::collections::HashMap;

pub struct UTxO {
    tx_id: Vec<u8>,
    index: u32,
    address: Vec<u8>,
    datum: Datum,
    reference_script: Option<Vec<u8>>,
    value: HashMap<String, u64>,
}

pub enum Datum {
    DatumHash(Vec<u8>),
    InlineDatum(Vec<u8>),
    None,
}
