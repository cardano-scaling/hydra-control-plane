use crate::model::hydra::tx::head_parameters::HeadParameters;
use crate::model::hydra::utxo::UTxO;

struct InitTx {
    network_id: u8,
    seed_input: UTxO, // Does this need to be a UTxO or can it just be an OutputRef?
    participants: Vec<u8>,
    paramters: HeadParameters,
}

impl InitTx {
    fn build_tx() {}
    pub fn to_bytes() -> Vec<u8> {
        todo!()
    }
}
