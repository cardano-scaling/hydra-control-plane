use std::collections::HashMap;

use pallas::{
    codec::minicbor::encode,
    ledger::{
        addresses::Address,
        primitives::conway::{BigInt, Constr, PlutusData, PolicyId},
    },
    txbuilder::Output,
};
use serde_json::Value;

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

#[derive(Debug, Clone)]
pub struct UTxO {
    hash: Vec<u8>,
    index: u32,
    address: String,
    datum: Datum,
    reference_script: Option<Script>,
    value: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub enum Datum {
    DatumHash(Vec<u8>),
    InlineDatum(PlutusData),
    None,
}

impl UTxO {
    pub fn try_from_value(tx_id: &str, value: &Value) -> Result<Self, Box<dyn std::error::Error>> {
        let index = tx_id.split("#").collect::<Vec<&str>>()[1].parse::<u32>()?;
        let hex_hash = tx_id.split("#").collect::<Vec<&str>>()[0];
        let hash = hex::decode(hex_hash)?;
        let address = value["address"]
            .as_str()
            .ok_or("Invalid address")?
            .to_string();
        let is_inline = !value["inlineDatum"].is_null();
        let is_hash = !value["datumHash"].is_null();
        let datum = if is_inline {
            Datum::InlineDatum(value_to_plutus_data(&value["inlineDatum"])?)
        } else if is_hash {
            Datum::DatumHash(hex::decode(
                value["datumHash"].as_str().ok_or("Invalid datumHash")?,
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
        for (key, value) in value["value"].as_object().ok_or("Invalid value")? {
            value_map.insert(key.to_string(), value.as_u64().ok_or("Invalid value")?);
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

fn value_to_plutus_data(value: &Value) -> Result<PlutusData, Box<dyn std::error::Error>> {
    let value = value
        .as_object()
        .ok_or("Invalid PlutusData json encoding")?;
    if value.contains_key("constructor") {
        let constructor = value
            .get("constructor")
            .ok_or("invalid constructor")?
            .as_u64()
            .ok_or("Invalid constructor")?;
        let fields: Vec<PlutusData> = value["fields"]
            .as_array()
            .ok_or("Invalid fields")?
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
            .ok_or("invalid int")?
            .as_i64()
            .ok_or("Invalid int")?;
        let big_int: BigInt = BigInt::Int(int.into());
        Ok(PlutusData::BigInt(big_int))
    } else if value.contains_key("bytes") {
        let bytes = value
            .get("bytes")
            .ok_or("invalid bytes")?
            .as_str()
            .ok_or("Invalid bytes")?;
        let bytes = hex::decode(bytes)?;
        Ok(PlutusData::BoundedBytes(bytes.into()))
    } else if value.contains_key("list") {
        Err("plutus list decoding not yet implemented".into())
    } else {
        Err("Invalid PlutusData json encoding".into())
    }
}
impl TryInto<Output> for UTxO {
    type Error = Box<dyn std::error::Error>;

    fn try_into(self) -> Result<Output, Self::Error> {
        let address = Address::from_bech32(self.address.as_str())?;
        let lovelace: u64 = self
            .value
            .get("lovelace")
            .unwrap_or(&u64::default())
            .clone();

        let mut output = Output::new(address, lovelace);
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
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let value = value.as_object().ok_or("invalid referenceScript object")?["script"]
            .as_object()
            .ok_or("invalid script object")?;

        let cbor = hex::decode(value["cborHex"].as_str().ok_or("invalid cborHex")?)?;
        let script_type: ScriptType = value["type"]
            .as_str()
            .ok_or("invalid scriptType")?
            .try_into()?;

        Ok(Script { cbor, script_type })
    }
}

impl TryFrom<&str> for ScriptType {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "PlutusScriptV1" => Ok(ScriptType::PlutusV1),
            "PlutusScriptV2" => Ok(ScriptType::PlutusV2),
            "NativeScript" => Ok(ScriptType::NativeScript),
            _ => Err("Invalid ScriptType".into()),
        }
    }
}
