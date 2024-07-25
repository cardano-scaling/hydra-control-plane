use anyhow::{anyhow, Context, Result};
use derivative::Derivative;
use pallas::{
    codec::minicbor::encode,
    crypto::hash::Hash,
    ledger::{
        addresses::Address,
        primitives::conway::{BigInt, Constr, PlutusData, PolicyId},
    },
    txbuilder::{Input, Output},
};
use serde_json::Value;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Script {
    cbor: Vec<u8>,
    script_type: ScriptType,
}

#[derive(Debug, Clone)]
pub enum ScriptType {
    PlutusV1,
    PlutusV2,
    NativeScript,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct UTxO {
    #[derivative(Debug(format_with = "crate::model::format_hex"))]
    pub hash: Vec<u8>,
    pub index: u64,
    pub address: Address,
    datum: Datum,
    pub reference_script: Option<Script>,
    pub value: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub enum Datum {
    DatumHash(Vec<u8>),
    InlineDatum(PlutusData),
    None,
}

impl UTxO {
    pub fn try_from_value(tx_id: &str, value: &Value) -> Result<Self> {
        let index = tx_id.split("#").collect::<Vec<&str>>()[1].parse::<u64>()?;
        let hex_hash = tx_id.split("#").collect::<Vec<&str>>()[0];
        let hash = hex::decode(hex_hash)?;
        let address = value["address"].as_str().context("Invalid address")?;
        let address = Address::from_bech32(address)?;
        let is_inline = !value["inlineDatum"].is_null();
        let is_hash = !value["datumHash"].is_null();
        let datum = if is_inline {
            Datum::InlineDatum(value_to_plutus_data(&value["inlineDatum"])?)
        } else if is_hash {
            Datum::DatumHash(hex::decode(
                value["datumHash"].as_str().context("Invalid datumHash")?,
            )?)
        } else {
            Datum::None
        };

        let reference_script: Option<Script> = if let Some(v) = value.get("referenceScript") {
            if v.is_null() {
                None
            } else {
                Some(v.try_into()?)
            }
        } else {
            None
        };

        let mut value_map = HashMap::new();
        for (key, value) in value["value"].as_object().context("Invalid value")? {
            value_map.insert(key.to_string(), value.as_u64().context("Invalid value")?);
        }

        Ok(UTxO {
            hash,
            index,
            address,
            datum,
            reference_script,
            value: value_map,
        })
    }
}

impl ToString for UTxO {
    fn to_string(&self) -> String {
        format!("{}#{}", hex::encode(&self.hash), self.index.to_string())
    }
}
impl Into<Input> for UTxO {
    fn into(self) -> Input {
        let hash: Hash<32> = self.hash.as_slice().try_into().unwrap();
        Input::new(hash, self.index)
    }
}

fn value_to_plutus_data(value: &Value) -> Result<PlutusData> {
    let value = value
        .as_object()
        .context("Invalid PlutusData json encoding")?;
    if value.contains_key("constructor") {
        let constructor = value
            .get("constructor")
            .context("key constructor not found")?
            .as_u64()
            .context("Invalid constructor")?;
        let fields: Vec<PlutusData> = value["fields"]
            .as_array()
            .context("Invalid fields")?
            .iter()
            .filter_map(|v| value_to_plutus_data(v).ok())
            .collect();

        Ok(PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: Some(constructor),
            fields,
        }))
    } else if value.contains_key("int") {
        let int = value
            .get("int")
            .context("key int not found")?
            .as_i64()
            .context("Invalid int")?;
        let big_int: BigInt = BigInt::Int(int.into());
        Ok(PlutusData::BigInt(big_int))
    } else if value.contains_key("bytes") {
        let bytes = value
            .get("bytes")
            .context("key bytes not found")?
            .as_str()
            .context("Invalid string")?;
        let bytes = hex::decode(bytes)?;
        Ok(PlutusData::BoundedBytes(bytes.into()))
    } else if value.contains_key("list") {
        Err(anyhow!("plutus list decoding not yet implemented"))
    } else {
        Err(anyhow!("Invalid PlutusData json encoding"))
    }
}
impl TryInto<Output> for UTxO {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Output, Self::Error> {
        let lovelace: u64 = self
            .value
            .get("lovelace")
            .unwrap_or(&u64::default())
            .clone();

        let mut output = Output::new(self.address, lovelace);
        for (asset_id, count) in self.value {
            if asset_id == "lovelace" {
                continue;
            }

            let asset_id = hex::decode(asset_id)?;
            let policy_id: [u8; 28] = asset_id[0..28].try_into()?;
            let policy_id: PolicyId = policy_id.into();
            let asset_name = asset_id[28..].to_vec();
            output = output.add_asset(policy_id, asset_name, count)?;
        }

        match self.datum {
            Datum::DatumHash(datum) => {
                let bytes: [u8; 32] = datum.try_into().unwrap();
                output = output.set_datum_hash(bytes.into());
            }
            Datum::InlineDatum(datum) => {
                let mut bytes: Vec<u8> = Vec::new();
                encode(datum, &mut bytes)?;
                output = output.set_inline_datum(bytes);
            }
            _ => {}
        }

        Ok(output)
    }
}

impl TryFrom<&Value> for Script {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let value = value
            .as_object()
            .context("invalid referenceScript object")?["script"]
            .as_object()
            .context("invalid script object")?;

        let cbor = hex::decode(value["cborHex"].as_str().context("invalid cborHex")?)?;
        let script_type: ScriptType = value["type"]
            .as_str()
            .context("invalid scriptType")?
            .try_into()?;

        Ok(Script { cbor, script_type })
    }
}

impl TryFrom<&str> for ScriptType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "PlutusScriptV1" => Ok(ScriptType::PlutusV1),
            "PlutusScriptV2" => Ok(ScriptType::PlutusV2),
            "NativeScript" => Ok(ScriptType::NativeScript),
            _ => Err(anyhow!("Invalid ScriptType")),
        }
    }
}
