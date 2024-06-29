use pallas::txbuilder::BuiltTransaction;
use serde::{ser::SerializeStruct, Serialize, Serializer};

pub struct NewTx {
    transaction: Transaction,
}

struct Transaction {
    cbor_hex: String,
    tx_id: String,
}

impl NewTx {
    pub fn new(tx: BuiltTransaction) -> Result<Self, Box<dyn std::error::Error>> {
        if tx.signatures.is_none() {
            return Err("No signatures".into());
        }

        Ok(NewTx {
            transaction: {
                Transaction {
                    cbor_hex: hex::encode(tx.tx_bytes),
                    tx_id: hex::encode(tx.tx_hash.0),
                }
            },
        })
    }
}

impl Serialize for NewTx {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("NewTx", 2)?;
        s.serialize_field("tag", "NewTx")?;
        s.serialize_field("transaction", &self.transaction)?;
        s.end()
    }
}

impl Serialize for Transaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("transaction", 4)?;
        s.serialize_field("type", "Witnessed Tx BabbageEra")?;
        s.serialize_field("description", "")?;
        s.serialize_field("cborHex", &self.cbor_hex)?;
        s.serialize_field("txId", &self.tx_id)?;

        s.end()
    }
}

impl Into<String> for NewTx {
    fn into(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
