use anyhow::{anyhow, Context, Result};

use pallas::{
    codec::{minicbor::encode, utils::MaybeIndefArray},
    crypto::hash::Hash,
    ledger::{
        addresses::PaymentKeyHash,
        primitives::conway::{Constr, PlutusData},
    },
    txbuilder::{BuildBabbage, BuiltTransaction, ExUnits, Output, StagingTransaction},
};

use crate::model::hydra::contract::hydra_validator::HydraValidator;

use super::{
    cost_models::COST_MODEL_PLUTUS_V2, input::InputWrapper, output::OutputWrapper,
    script_registry::ScriptRegistry,
};

pub struct CommitTx {
    pub network_id: u8,
    pub script_registry: ScriptRegistry,
    pub head_id: Vec<u8>,
    pub party: Vec<u8>,
    pub initial_input: (InputWrapper, Output, PaymentKeyHash),
    pub blueprint_tx: Vec<(InputWrapper, OutputWrapper)>,
    pub fee: u64,
    pub commit_inputs: Vec<(InputWrapper, OutputWrapper)>,
}

impl CommitTx {
    pub fn build_tx(&self) -> Result<BuiltTransaction> {
        let commit_output = build_base_commit_output(
            [
                self.commit_inputs
                    .iter()
                    .map(|(_, o)| o.inner.clone())
                    .collect::<Vec<Output>>()
                    .as_slice(),
                vec![self.initial_input.1.clone()].as_slice(),
            ]
            .concat(),
            self.network_id,
        )
        .context("Failed to construct base commit output")?
        .set_inline_datum(self.build_commit_datum()?);

        let mut tx_builder = Some(
            StagingTransaction::new()
                .fee(self.fee)
                .reference_input(self.script_registry.initial_reference.clone().into())
                .collateral_input(
                    self.blueprint_tx
                        .get(0)
                        .ok_or(anyhow!(
                            "need at least one blueprint tx input for collateral"
                        ))?
                        .0
                        .clone()
                        .into(),
                )
                .input(self.initial_input.0.clone().into())
                .output(commit_output)
                .add_spend_redeemer(
                    self.initial_input.0.clone().into(),
                    self.build_redeemer()?,
                    Some(ExUnits {
                        mem: 14000000,
                        steps: 10000000000,
                    }),
                )
                .disclosed_signer(self.initial_input.2)
                .language_view(
                    pallas::txbuilder::ScriptKind::PlutusV2,
                    COST_MODEL_PLUTUS_V2.clone(),
                ),
        );
        for (input, _) in self.commit_inputs.clone() {
            if let Some(builder) = tx_builder {
                tx_builder = Some(builder.input(input.into()));
            }
        }

        for (input, output) in self.blueprint_tx.clone() {
            if let Some(builder) = tx_builder {
                tx_builder = Some(builder.input(input.into()).output(output.inner));
            }
        }

        tx_builder
            .ok_or(anyhow!("no transaction builder "))
            .and_then(|builder| builder.build_babbage_raw().map_err(|e| anyhow!("{}", e)))
            .map_err(|e| anyhow!("failed to build tx: {}", e))
    }

    fn build_commit_datum(&self) -> Result<Vec<u8>> {
        let data = PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: MaybeIndefArray::Indef(vec![
                PlutusData::BoundedBytes(self.party.clone().into()),
                PlutusData::Array(MaybeIndefArray::Indef(
                    self.commit_inputs
                        .clone()
                        .into_iter()
                        .map(|(commit_input, commit_input_output)| {
                            let output_data: PlutusData = commit_input_output.into();
                            let mut output_bytes = Vec::new();
                            encode(&output_data, &mut output_bytes)?;
                            Ok(PlutusData::Constr(Constr {
                                tag: 121,
                                any_constructor: None,
                                fields: MaybeIndefArray::Indef(vec![
                                    commit_input.into(),
                                    PlutusData::BoundedBytes(output_bytes.into()),
                                ]),
                            }))
                        })
                        .collect::<Result<Vec<PlutusData>, anyhow::Error>>()?,
                )),
                PlutusData::BoundedBytes(self.head_id.clone().into()),
            ]),
        });

        let mut bytes: Vec<u8> = Vec::new();
        encode(&data, &mut bytes).context("Failed to encode plutus data in CBOR")?;

        Ok(bytes)
    }

    fn build_redeemer(&self) -> Result<Vec<u8>> {
        let redeemer_data = PlutusData::Constr(Constr {
            tag: 122,
            any_constructor: None,
            fields: MaybeIndefArray::Indef(vec![PlutusData::Array(MaybeIndefArray::Indef(
                self.commit_inputs
                    .iter()
                    .map(|(input, _)| input.into())
                    .collect::<Vec<_>>(),
            ))]),
        });

        let mut bytes: Vec<u8> = Vec::new();
        encode(&redeemer_data, &mut bytes).context("Failed to encode plutus data in CBOR")?;
        Ok(bytes)
    }
}

fn build_base_commit_output(outputs: Vec<Output>, network_id: u8) -> Result<Output> {
    let address = HydraValidator::VCommit.to_address(network_id);
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
    use pallas::{crypto::hash::Hash, ledger::addresses::Address, txbuilder::Input};

    use crate::model::hydra::tx::script_registry::NetworkScriptRegistry;

    use super::*;

    #[test]
    fn test_build_commit_datum() {
        let datum = get_commit()
            .build_commit_datum()
            .expect("Failed to build commit datum");

        assert_eq!(hex::encode(datum), "d8799f58203302e982ae2514964bcd2b2d7187277a2424e44b553efafaf786677ff5db9a5e9fd8799fd8799fd8799f582008e378358bffd92fc354ee757b5c47204ba58e7c72347a08877abab5ba202948ff182eff5f5840d8799fd8799fd8799f581c299650d431de775c65eed15c122aa975237e5b4a235a596c0b5edcf3ffd8799fd8799fd8799f581c496ab2039877b6386666a3d6515823e38eaf04c7c0a46c09f7f939ebfd6effffffffa140a1401a00989680d87980d87a80ffffffd8799fd8799fd8799f58205a41c22049880541a23954877bd2e5e6069b5ecb8eed6505dbf16f5ee45e9fa8ff03ff5f5840d8799fd8799fd8799f581cb9343d7c6de66302960110db759633e7bc4ce1ef8d3faa2386938dedffd8799fd8799fd8799f581c8202e1e5de55b5025e8a4afe4558239cea10057e97825c3d34623ad28d1fffffffffa140a1401a05c81a40d87980d87a80ffffffd8799fd8799fd8799f58207663bc29c18d4d3647ff6f5054815c2b5f0fd76fafd1e6f5613f7471a88d8fa0ff07ff5f5840d8799fd8799fd8799f581cee1a6e03cbd9ede8d40ae6fdb6ab51a9de7b603af218730fe5e56d35ffd8799fd8799fd8799f581c504be8610c412878fb01c9ef7758239f1e6bbd1c1aeafe3c6c6c8f50d142ffffffffa140a1401a030a32c0d87980d87a80ffffffff581c2505642019121d9b2d92437d8b8ea493bacfcb4fb535013b70e7f528ff");
    }

    #[test]
    fn test_build_redeemer() {
        let redeemer = get_commit()
            .build_redeemer()
            .expect("Failed to build redeemer");

        assert_eq!(hex::encode(redeemer), "d87a9f9fd8799fd8799f582008e378358bffd92fc354ee757b5c47204ba58e7c72347a08877abab5ba202948ff182effd8799fd8799f58205a41c22049880541a23954877bd2e5e6069b5ecb8eed6505dbf16f5ee45e9fa8ff03ffd8799fd8799f58207663bc29c18d4d3647ff6f5054815c2b5f0fd76fafd1e6f5613f7471a88d8fa0ff07ffffff");
    }

    // TODO: we need to actually build a test that works here
    #[test]
    fn test_build_tx() {
        let commit = build_preprod_commit();
        let tx = commit.build_tx().expect("Failed to build tx");

        println!("{:?}", hex::encode(tx.tx_bytes));

        assert!(true);
    }

    fn build_preprod_commit() -> CommitTx {
        let head_id = hex::decode("bfab6b5ece7eba6d4cdde8cfc5e0f91ac8a097c90b14d7eb934126da")
            .expect("Failed to decode head_id");
        let party = hex::decode("7bbfc8ffc6da9e6f6f070f0f28a4c0de8e099c34485e192660475059d8bb9557")
            .expect("Failed to decode party");

        let initial_input: (InputWrapper, Output, PaymentKeyHash) = (
            Input::new(
                Hash::from(
                    hex::decode("12b552763c92793685bafc8854112d2868373bafa03b1f011dbdb426dc226fc8")
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
                2000000,
            )
            .add_asset(
                Hash::from(
                    hex::decode("bfab6b5ece7eba6d4cdde8cfc5e0f91ac8a097c90b14d7eb934126da")
                        .expect("failed to decode policy id")
                        .as_slice(),
                ),
                hex::decode("9b29dd55b38a5e824775d303723e83e08d83c4ba72ceab284154b8a2")
                    .expect("failed to decode asset id"),
                1,
            )
            .expect("failed to add asset to initial output"),
            Hash::from(
                hex::decode("9b29dd55b38a5e824775d303723e83e08d83c4ba72ceab284154b8a2")
                    .expect("failed to decode key hash")
                    .as_slice(),
            ),
        );

        CommitTx {
            network_id: 0,
            script_registry: NetworkScriptRegistry::Preprod.into(),
            head_id,
            party,
            initial_input,
            blueprint_tx: vec![(
                Input::new(
                    Hash::from(
                        hex::decode(
                            "12b552763c92793685bafc8854112d2868373bafa03b1f011dbdb426dc226fc8",
                        )
                        .expect("failed to decode tx_id")
                        .as_slice(),
                    ),
                    2,
                )
                .into(),
                Output::new(
                    Address::from_bech32(
                        "addr_test1vzdjnh24kw99aqj8whfsxu37s0sgmq7yhfeva2egg92t3gsws2hwn",
                    )
                    .expect("failed to decode bech32 address"),
                    9974285986 - 1875229,
                )
                .into(),
            )],
            fee: 1875229,
            commit_inputs: vec![(
                Input::new(
                    Hash::from(
                        hex::decode(
                            "4991e003de580e917c5ab659f7c6d054c0827e6fc30695351d6d9c13adb44c0c",
                        )
                        .expect("failed to decode tx_id")
                        .as_slice(),
                    ),
                    0,
                )
                .into(),
                Output::new(
                    Address::from_bech32(
                        "addr_test1vzdjnh24kw99aqj8whfsxu37s0sgmq7yhfeva2egg92t3gsws2hwn",
                    )
                    .expect("failed to decode bech32 address"),
                    10000000000,
                )
                .into(),
            )],
        }
    }

    // This CommitTx uses the following preview transaction: d00b6b2c3920c8836ca0bce2fe4f662bd68c3d49dca743831fd9328b44260908
    fn get_commit() -> CommitTx {
        let head_id = hex::decode("2505642019121D9B2D92437D8B8EA493BACFCB4FB535013B70E7F528")
            .expect("Failed to decode head_id");
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
            )
            .add_asset(
                Hash::from(
                    hex::decode("2505642019121D9B2D92437D8B8EA493BACFCB4FB535013B70E7F528")
                        .expect("failed to decode policy id")
                        .as_slice(),
                ),
                hex::decode("8BB334F0E8D88551D62DB31965F25B644FE0CCC8D3613533E10D689A")
                    .expect("failed to decode asset id"),
                1,
            )
            .expect("failed to add asset to initial output"),
            Hash::from(
                hex::decode("8BB334F0E8D88551D62DB31965F25B644FE0CCC8D3613533E10D689A")
                    .expect("failed to decode key hash")
                    .as_slice(),
            ),
        );

        CommitTx {
                network_id: 0,
                script_registry: NetworkScriptRegistry::Preprod.into(),
                head_id,
                party,
                initial_input,
                blueprint_tx: vec![(Input::new(Hash::from(hex::decode("ef61c1686e77e6004f7e9913d20d0598e8cc5e661a559086a84dfafaafdc7818").expect("failed to decode tx_id").as_slice()), 2).into(), Output::new(Address::from_bech32("addr_test1vz9mxd8sarvg25wk9ke3je0jtdjylcxverfkzdfnuyxk3xszsdn9j").expect("failed to decode bech32 address"), 917935379).into())],
                fee: 1822653,
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
                    ).into(),
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
                    ).into(),
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
                    ).into()
                )
                ],
            }
    }
}
