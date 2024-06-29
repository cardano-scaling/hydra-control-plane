use std::hash::Hash;

use pallas::{
    crypto::key::ed25519::SecretKey,
    ledger::addresses::{Address, ShelleyPaymentPart},
    txbuilder::{BuildBabbage, BuiltTransaction, Input, Output, StagingTransaction},
};

use hex::FromHex;

use super::hydra::utxo::UTxO;

pub struct TxBuilder {
    admin_key: ShelleyPaymentPart,
    script_ref: Option<UTxO>,
}

impl TxBuilder {
    pub fn new(admin_key: ShelleyPaymentPart, script_ref: Option<UTxO>) -> Self {
        TxBuilder {
            admin_key,
            script_ref,
        }
    }
}

pub fn build_tx() -> BuiltTransaction {
    let hash: [u8; 32] = [
        204, 200, 153, 252, 40, 49, 85, 188, 162, 75, 210, 191, 138, 156, 80, 51, 128, 155, 218,
        188, 217, 108, 26, 193, 3, 47, 193, 77, 91, 238, 215, 71,
    ];

    let address =
        Address::from_bech32("addr_test1vqx5tu4nzz5cuanvac4t9an4djghrx7hkdvjnnhstqm9kegvm6g6c")
            .unwrap();

    let sk: SecretKey =
        <[u8; 32]>::from_hex("0E3F3546A93BD1295EB9DCE216941EEFBCE99CA9323DF258D9BEEEE335920CCE")
            .unwrap()
            .into();

    let tx = StagingTransaction::new()
        .input(Input::new(hash.into(), 250000000))
        .output(Output::new(address, 250000000))
        .fee(0)
        .build_babbage_raw()
        .unwrap();

    tx.sign(sk.into()).unwrap()
}
