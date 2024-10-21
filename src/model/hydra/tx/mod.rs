use pallas::{
    codec::minicbor::encode,
    ledger::primitives::conway::{Constr, PlutusData},
};

pub mod commit;
pub mod head_parameters;
pub mod init;
pub mod input;
pub mod output;
pub mod script_registry;

pub fn void_redeemer() -> Vec<u8> {
    let data = PlutusData::Constr(Constr {
        tag: 121,
        any_constructor: None,
        fields: vec![],
    });

    let mut bytes: Vec<u8> = Vec::new();
    encode(&data, &mut bytes).unwrap();

    bytes
}
