use serde_json::{Map, Value};

use crate::error::Result;
use crate::model::{DataTransaction, SignedTransaction, Transaction, TransactionData};
use crate::util::Base58;

pub struct JsonSerializer;

impl JsonSerializer {
    pub fn serialize_signed_tx(sign_tx: &SignedTransaction) -> Result<Value> {
        let mut json_props: Map<String, Value> = Map::new();
        let mut json_props_with_default_values = add_default_fields(sign_tx, &mut json_props)?;
        let json_props_with_additional_fields =
            add_additional_fields(sign_tx.tx().data(), &mut json_props_with_default_values)?;
        Ok(json_props_with_additional_fields.into())
    }
}

fn add_default_fields(
    sign_tx: &SignedTransaction,
    json_props: &mut Map<String, Value>,
) -> Result<Map<String, Value>> {
    let tx = sign_tx.tx();
    json_props.insert("type".to_string(), tx_type(tx).into());
    json_props.insert("version".to_string(), tx.version().into());
    json_props.insert("chainId".to_string(), tx.chain_id().into());
    json_props.insert(
        "senderPublicKey".to_string(),
        tx.public_key().encoded().into(),
    );
    json_props.insert(
        "sender".to_string(),
        tx.public_key()
            .address(sign_tx.tx().chain_id())?
            .encoded()
            .into(),
    );
    json_props.insert("fee".to_string(), tx.fee().fee().into());
    json_props.insert("feeAssetId".to_string(), tx.fee().fee_asset_id().into());
    json_props.insert("timestamp".to_string(), tx.timestamp().into());
    json_props.insert("proofs".to_string(), proofs(sign_tx).into());
    Ok(json_props.clone())
}

fn add_additional_fields(
    tx_data: &TransactionData,
    json_props: &mut Map<String, Value>,
) -> Result<Map<String, Value>> {
    match tx_data {
        TransactionData::Transfer(_) => todo!(),
        TransactionData::Data(data_tx) => {
            json_props.insert("data".to_string(), data_tx.data().into())
        }
    };
    Ok(json_props.clone())
}

fn tx_type(tx: &Transaction) -> u8 {
    match tx.data() {
        TransactionData::Transfer(_) => todo!(),
        TransactionData::Data(_) => DataTransaction::tx_type(),
    }
}

fn proofs(sign_tx: &SignedTransaction) -> Vec<String> {
    sign_tx
        .proofs()
        .iter()
        .map(|proof| Base58::encode(proof, false))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::model::account::PublicKey;
    use crate::model::data_entry::DataEntry;
    use crate::model::{
        Amount, ChainId, DataTransaction, SignedTransaction, Transaction, TransactionData,
    };
    use crate::util::json::json_deserializer::JsonDeserializer;
    use crate::util::{Base58, JsonSerializer};

    #[test]
    fn test_data_transaction_to_json() {
        let binary_value: [u8; 12] = [0; 12];

        let transaction_data = TransactionData::Data(DataTransaction::new(vec![
            DataEntry::IntegerEntry {
                key: "int".to_string(),
                value: 12,
            },
            DataEntry::BooleanEntry {
                key: "bool".to_string(),
                value: false,
            },
            DataEntry::BinaryEntry {
                key: "binary".to_string(),
                value: binary_value.to_vec(),
            },
            DataEntry::StringEntry {
                key: "str".to_string(),
                value: "value".to_string(),
            },
        ]));

        let public_key = PublicKey::from_string("8jDzNuHZwuTTo6WvZMdSoNc8ydY6a7UnxvwHZ8kooMuS")
            .expect("failed to get public key from string");
        let signed_transaction = SignedTransaction::new(
            Transaction::new(
                transaction_data,
                Amount::new(100000, None),
                1661456063029,
                public_key,
                DataTransaction::tx_type(),
                2,
                ChainId::TESTNET.byte(),
            ),
            vec![
                Base58::decode(
                "4nDUCnVw9j9D5bTBSLfFCHR9CtvS32mSdxctccChRAohfLwz3ng3ps5ffUiy4NtRmXG7vDHRMW57ABxzkMW64tzC"
            ).expect("Failed to decode base58 string")
            ]
        );

        let json = JsonSerializer::serialize_signed_tx(&signed_transaction)
            .expect("failed to serialize signed transaction");
        let signed_tx_from_json =
            JsonDeserializer::deserialize_signed_tx(&json, ChainId::TESTNET.byte())
                .expect("Failed to deserialize signed tx");
        assert_eq!(signed_transaction, signed_tx_from_json);
    }
}
