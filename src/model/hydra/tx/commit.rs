use anyhow::{Context, Result};

use pallas::{
    codec::{minicbor::encode, utils::CborWrap},
    ledger::{
        addresses::PaymentKeyHash,
        primitives::{
            conway::{
                Constr, DatumOption, NativeScript, PlutusData, PlutusV1Script, PlutusV2Script,
                PostAlonzoTransactionOutput, PseudoScript, PseudoTransactionOutput, Value,
            },
            Fragment,
        },
    },
    txbuilder::{Output, ScriptKind, StagingTransaction, TxBuilderError},
};

use crate::model::hydra::contract::hydra_validator::HydraValidator;

use super::{input::InputWrapper, script_registry::ScriptRegistry};

pub struct CommitTx {
    network_id: u8,
    script_registry: ScriptRegistry,
    head_id: Vec<u8>,
    party: Vec<u8>,
    initial_input: (InputWrapper, Output, PaymentKeyHash),
    commit_inputs: Vec<(InputWrapper, Output)>,
}

impl CommitTx {
    pub fn build_tx(&self) -> Result<StagingTransaction> {
        let commit_output = build_base_commit_output(
            self.commit_inputs.iter().map(|(_, o)| o.clone()).collect(),
            self.network_id,
        )
        .context("Failed to construct base commit output")?
        .set_inline_datum(self.make_commit_datum()?);

        let tx_builder = StagingTransaction::new().output(commit_output);

        Ok(tx_builder)
    }

    fn make_commit_datum(&self) -> Result<Vec<u8>> {
        let data = PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![
                PlutusData::BoundedBytes(self.party.clone().into()),
                PlutusData::Array(
                    self.commit_inputs
                        .clone()
                        .into_iter()
                        .map(|(commit_input, commit_input_output)| {
                            // let conway_output = commit_input_output.build_babbage_raw()?;
                            // let mut output_bytes = Vec::new();
                            // encode(&conway_output, &mut output_bytes)?;
                            PlutusData::Constr(Constr {
                                tag: 121,
                                any_constructor: None,
                                fields: vec![
                                    commit_input.into(),
                                    // PlutusData::BoundedBytes(output_bytes.into()),
                                ],
                            })
                        })
                        .collect(),
                ),
            ],
        });

        let mut bytes: Vec<u8> = Vec::new();
        encode(&data, &mut bytes).context("Failed to encode plutus data in CBOR")?;

        Ok(bytes)
    }
}

fn build_base_commit_output(outputs: Vec<Output>, network_id: u8) -> Result<Output> {
    let address = HydraValidator::VDeposit.to_address(network_id);
    let lovelace = outputs.iter().fold(0, |acc, o| acc + o.lovelace);
    let mut commit_output = Output::new(address, lovelace);
    for output in outputs {
        if let Some(output_assets) = output.assets {
            for (policy, assets) in output_assets.iter() {
                for (name, amount) in assets {
                    commit_output = commit_output
                        .add_asset(policy.0.into(), name.0.clone(), amount.clone())
                        .context("Failed to add asset to commit output")?;
                }
            }
        }
    }

    Ok(commit_output)
}

mod tests {
    use pallas::{
        codec::minicbor::decode, crypto::hash::Hash, ledger::addresses::Address, txbuilder::Input,
    };

    use super::*;

    #[test]
    fn test_make_commit_datum() {
        let head_id = hex::decode("d8799fd8799fd8799f581c299650d431de775c65eed15c122aa975237e5b4a235a596c0b5edcf3ffd8799fd8799fd8799f581c496ab2039877b6386666a3d651e38eaf04c7c0a46c09f7f939ebfd6effffffffa140a1401a00989680d87980d87a80ff").expect("Failed to decode head_id");
        let party = hex::decode("3302e982ae2514964bcd2b2d7187277a2424e44b553efafaf786677ff5db9a5e")
            .expect("Failed to decode party");
        let initial_input: (InputWrapper, Output, PaymentKeyHash) = (
            Input::new(
                Hash::from(
                    hex::decode("ef61c1686e77e6004f7e9913d20d0598e8cc5e661a559086a84dfafaafdc7818")
                        .expect("Failed to decode txid")
                        .as_slice(),
                ),
                1,
            )
            .into(),
            Output::new(
                Address::from_bech32(
                    "addr_test1wqh6eqv6ra83fc5k88g5zs3q62sck64adw8ygnvg6rw63lc70pepc",
                )
                .expect("failed to decode bech32"),
                1290000,
            ),
            Hash::from(
                hex::decode("2fac819a1f4f14e29639d1414220d2a18b6abd6b8e444d88d0dda8ff")
                    .expect("failed to decode key hash")
                    .as_slice(),
            ),
        );

        let commit: CommitTx = CommitTx {
            network_id: 0,
            script_registry: ScriptRegistry {
                initial_reference: initial_input.0.clone().into(),
                commit_reference: initial_input.0.clone().into(),
                head_reference: initial_input.0.clone().into(),
            },
            head_id,
            party,
            initial_input,
            commit_inputs: vec![(
                Input::new(
                    Hash::from(
                        hex::decode(
                            "08e378358bffd92fc354ee757b5c47204ba58e7c72347a08877abab5ba202948",
                        )
                        .expect("Failed to decode txid")
                        .as_slice(),
                    ),
                    46,
                )
                .into(),
                Output::new(
                    Address::from_bech32(
                        "addr_test1qq5ev5x5x808whr9amg4cy32496jxljmfg345ktvpd0deu6fd2eq8xrhkcuxve4r6eg78r40qnrupfrvp8mljw0tl4hqe383dk"
                    )
                    .expect("failed to decode bech32"),
                    10000000
                ),
            ),
            (
                Input::new(
                    Hash::from(
                        hex::decode(
                            "5a41c22049880541a23954877bd2e5e6069b5ecb8eed6505dbf16f5ee45e9fa8",
                        )
                        .expect("Failed to decode txid")
                        .as_slice(),
                    ),
                    3,
                )
                .into(),
                Output::new(
                    Address::from_bech32(
                        "addr_test1qzung0tudhnxxq5kqygdkavkx0nmcn8pa7xnl23rs6fcmmvzqts7thj4k5p9azj2lezee6ssq4lf0qju856xywkj350sew4adl"
                    )
                    .expect("failed to decode bech32"),
                    97000000
                ),
            ),
            (
                Input::new(
                    Hash::from(
                        hex::decode(
                            "7663bc29c18d4d3647ff6f5054815c2b5f0fd76fafd1e6f5613f7471a88d8fa0"
                        )
                        .expect("failed to decode tx_id")
                        .as_slice()
                    ),
                    7
                )
                .into(),
                Output::new(
                    Address::from_bech32(
                        "addr_test1qrhp5msre0v7m6x5ptn0md4t2x5au7mq8tepsuc0uhjk6d2sf05xzrzp9pu0kqwfaame78nth5wp46h783kxer6s69pq74eyeg"
                    )
                    .expect("failed to decode bech32"),
                    51000000
                )
            )
            ],
        };

        let datum = commit
            .make_commit_datum()
            .expect("Failed to make commit datum");

        let datum_string = hex::encode(datum);
        println!("{}", datum_string);

        assert!(true);
    }
}
