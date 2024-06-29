use std::collections::HashMap;

use pallas::{
    codec::utils::Bytes,
    ledger::{addresses::Address, primitives::conway::PolicyId},
    txbuilder::Output,
};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct UTxO {
    hash: Vec<u8>,
    index: u32,
    address: String,
    datum: Datum,
    reference_script: Option<Vec<u8>>,
    value: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub enum Datum {
    DatumHash(Vec<u8>),
    InlineDatum(Vec<u8>),
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
        let datum: Datum;
        if is_inline {
            datum = Datum::InlineDatum(hex::decode(
                value["inlineDatum"].as_str().ok_or("Invalid inlineDatum")?,
            )?);
        } else if is_hash {
            datum = Datum::DatumHash(hex::decode(
                value["datumHash"].as_str().ok_or("Invalid datumHash")?,
            )?);
        } else {
            datum = Datum::None;
        };

        let reference_script = if value["referenceScript"].is_null() {
            None
        } else {
            Some(hex::decode(
                value["referenceScript"]
                    .as_str()
                    .ok_or("Invalid referenceScript")?,
            )?)
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

// impl TryInto<Output> for UTxO {
//     type Error = Box<dyn std::error::Error>;

//     fn try_into(self) -> Result<Output, Self::Error> {
//         let address = Address::from_bech32(self.address.as_str())?;
//         let lovelace: u64 = self
//             .value
//             .get("lovelace")
//             .unwrap_or(&u64::default())
//             .clone();
//         let assets = self
//             .value
//             .into_iter()
//             .filter(|(key, _)| key != "lovelace")
//             .map(|(key, value)| {
//                 let result = hex::decode(key);
//                 if result.is_err() {
//                     return result.err().unwrap();
//                 }
//             })
//             .collect::<Result<HashMap<PolicyId, HashMap<Bytes, u64>>, Box<dyn std::error::Error>>>(
//             )?;
//         // Output {
//         //     address,
//         //     lovelace,
//         //     value: self.value,
//         //     datum: self.datum,
//         //     reference_script: self.reference_script,
//         // }
//         todo!()
//     }
// }
