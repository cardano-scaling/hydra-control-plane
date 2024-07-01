use pallas::{
    crypto::key::ed25519::SecretKey,
    ledger::addresses::Address,
    txbuilder::{BuildBabbage, BuiltTransaction, Input, Output, StagingTransaction},
};

use hex::FromHex;

use super::{hydra::utxo::UTxO, player::Player};

#[derive(Clone)]
pub struct TxBuilder {
    admin_key: SecretKey,
    script_ref: Option<Output>,
}

impl TxBuilder {
    pub fn new(admin_key: [u8; 32]) -> Self {
        let admin_key: SecretKey = admin_key.into();
        TxBuilder {
            admin_key,
            script_ref: None,
        }
    }

    pub fn set_script_ref(&mut self, script_ref: UTxO) -> Result<(), Box<dyn std::error::Error>> {
        let script_ref: Output = script_ref.try_into()?;
        self.script_ref = Some(script_ref);

        Ok(())
    }

    pub fn build_new_game_state(
        &self,
        player: Player,
    ) -> Result<BuiltTransaction, Box<dyn std::error::Error>> {
        if let None = self.script_ref {
            return Err("There must be a script reference in order to build game state".into());
        }
        unimplemented!()
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
        .input(Input::new(hash.into(), 0))
        .output(Output::new(address, 0))
        .fee(0)
        .build_babbage_raw()
        .unwrap();

    tx.sign(sk.into()).unwrap()
}
