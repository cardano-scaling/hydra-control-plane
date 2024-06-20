use std::error::Error;

use serde_json::Value;

#[derive(Debug)]
pub struct HeadIsInitializing {
    head_id: String,
    parties: Vec<Vec<u8>>,
    seq: u64,
    timestamp: String,
}

impl TryFrom<Value> for HeadIsInitializing {
    type Error = Box<dyn Error>;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let head_id = value["headId"]
            .as_str()
            .ok_or("Invalid head_id")?
            .to_owned();
        let parties_arr = value["parties"].as_array().ok_or("Invalid parties")?;
        let parties = parties_arr
            .into_iter()
            .map(|party| {
                let x = match party.as_object() {
                    Some(party_obj) => {
                        hex::decode(party_obj["vkey"].as_str().ok_or("Invalid vkey")?)
                            .map_err(|e| e.to_string())
                    }
                    None => Err("Invalid party object".into()),
                };

                x
            })
            .collect::<Result<Vec<Vec<u8>>, String>>()?;
        let seq = value["seq"].as_u64().ok_or("Invalid seq")?;
        let timestamp = value["timestamp"].as_str().ok_or("Invalid timestamp")?;

        Ok(HeadIsInitializing {
            head_id: head_id.to_string(),
            parties,
            seq,
            timestamp: timestamp.to_string(),
        })
    }
}
