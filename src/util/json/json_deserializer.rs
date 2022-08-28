use crate::errors::ParseError;
use crate::model::account::{Address, Balance, BalanceDetails};
use crate::model::data_entry::DataEntry;
use crate::model::TransactionData::{Data, Transfer};
use crate::model::{
    Amount, ApplicationStatus, DataTransaction, SignedTransaction, Transaction, TransactionInfo,
    TransferTransaction,
};
use crate::util::Base58;
use serde_json::Value;

pub struct JsonDeserializer;

impl JsonDeserializer {
    pub fn deserialize_tx_info(value: &Value, chain_id: u8) -> Result<TransactionInfo, ParseError> {
        let id = Self::safe_to_string_from_field(value, "id")?;

        let application_status =
            match Self::safe_to_string_from_field(value, "applicationStatus")?.as_str() {
                "succeeded" => ApplicationStatus::Succeed,
                //todo check statuses
                "scriptExecutionFailed" => ApplicationStatus::ScriptExecutionFailed,
                &_ => ApplicationStatus::Unknown,
            };
        let height = Self::safe_to_int_from_field(value, "height")? as u32;
        let signed_transaction = Self::deserialize_signed_tx(value, chain_id)?;

        Ok(TransactionInfo::new(
            id,
            signed_transaction,
            application_status,
            height,
        ))
    }

    pub fn deserialize_signed_tx(
        value: &Value,
        chain_id: u8,
    ) -> Result<SignedTransaction, ParseError> {
        let transaction = Self::deserialize_tx(value, chain_id)?;
        let proofs_array = Self::safe_to_array_from_field(value, "proofs")?;
        let proofs = proofs_array
            .iter()
            // todo remove unwrap
            .map(|v| Base58::decode(v.as_str().unwrap()).unwrap())
            .collect::<Vec<Vec<u8>>>();
        Ok(SignedTransaction::new(transaction, proofs))
    }

    pub fn deserialize_tx(value: &Value, chain_id: u8) -> Result<Transaction, ParseError> {
        let tx_type = Self::safe_to_int_from_field(value, "type")? as u8;
        let fee = Self::safe_to_int_from_field(value, "fee")? as u64;
        let fee_asset_id = value["feeAssetId"].as_str().map(|value| value.into());
        let transaction_data = match tx_type {
            4 => Transfer(TransferTransaction::from_json(value)),
            12 => Data(DataTransaction::from_json(value)),
            _ => panic!("unknown tx type"),
        };
        let timestamp = Self::safe_to_int_from_field(value, "timestamp")? as u64;
        let public_key = Self::safe_to_string_from_field(value, "senderPublicKey")?.try_into();
        let version = Self::safe_to_int_from_field(value, "version")? as u8;
        Ok(Transaction::new(
            transaction_data,
            Amount::new(fee, fee_asset_id),
            timestamp,
            public_key.unwrap(),
            tx_type,
            version,
            chain_id,
        ))
    }

    pub fn deserialize_addresses(value: &Value, chain_id: u8) -> Result<Vec<Address>, ParseError> {
        let array = Self::safe_to_array(value)?;
        array
            .iter()
            .map(|address| Self::deserialize_address(address, chain_id))
            .collect()
    }

    pub fn deserialize_address(value: &Value, chain_id: u8) -> Result<Address, ParseError> {
        let string = Self::safe_to_string(value)?;
        Ok(Address::from_string(&string, chain_id))
    }

    pub fn deserialize_balances(value: &Value, chain_id: u8) -> Result<Vec<Balance>, ParseError> {
        let array = Self::safe_to_array(value)?;
        array
            .iter()
            .map(|balance| Self::deserialize_balance(balance, chain_id))
            .collect()
    }

    pub fn deserialize_balance(value: &Value, chain_id: u8) -> Result<Balance, ParseError> {
        let address =
            Address::from_string(&Self::safe_to_string_from_field(value, "id")?, chain_id);
        let balance = Self::safe_to_int_from_field(value, "balance")?;
        Ok(Balance::new(address, balance as u64))
    }

    pub fn deserialize_balance_details(
        value: &Value,
        chain_id: u8,
    ) -> Result<BalanceDetails, ParseError> {
        let address = Address::from_string(
            &Self::safe_to_string_from_field(value, "address")?,
            chain_id,
        );
        let available = Self::safe_to_int_from_field(value, "available")? as u64;
        let regular = Self::safe_to_int_from_field(value, "regular")? as u64;
        let generating = Self::safe_to_int_from_field(value, "generating")? as u64;
        let effective = Self::safe_to_int_from_field(value, "effective")? as u64;
        Ok(BalanceDetails::new(
            address, available, regular, generating, effective,
        ))
    }

    pub fn deserialize_data_array(value: &Value) -> Result<Vec<DataEntry>, ParseError> {
        let data_array = Self::safe_to_array(value)?;
        Ok(data_array
            .iter()
            .map(|entry| entry.into())
            .collect::<Vec<DataEntry>>())
    }

    pub fn safe_to_string_from_field(json: &Value, field_name: &str) -> Result<String, ParseError> {
        let string = json[field_name]
            .as_str()
            .ok_or_else(|| ParseError::FieldNotFoundError {
                json: json.to_string(),
                field_name: field_name.to_owned(),
            })?;
        Ok(string.into())
    }

    pub fn safe_to_int_from_field(json: &Value, field_name: &str) -> Result<i64, ParseError> {
        let int = json[field_name]
            .as_i64()
            .ok_or_else(|| ParseError::FieldNotFoundError {
                json: json.to_string(),
                field_name: field_name.to_owned(),
            })?;
        Ok(int)
    }

    pub fn safe_to_array_from_field(
        json: &Value,
        field_name: &str,
    ) -> Result<Vec<Value>, ParseError> {
        let array = json[field_name]
            .as_array()
            .ok_or_else(|| ParseError::FieldNotFoundError {
                json: json.to_string(),
                field_name: field_name.to_owned(),
            })?;
        Ok(array.to_owned())
    }

    pub fn safe_to_string(json: &Value) -> Result<String, ParseError> {
        let string = json.as_str().ok_or_else(|| ParseError::InvalidTypeError {
            json: json.to_string(),
            json_type: "String".to_string(),
        })?;
        Ok(string.to_owned())
    }

    pub fn safe_to_int(json: &Value) -> Result<i64, ParseError> {
        let int = json.as_i64().ok_or_else(|| ParseError::InvalidTypeError {
            json: json.to_string(),
            json_type: "i64".to_string(),
        })?;
        Ok(int.to_owned())
    }

    pub fn safe_to_array(json: &Value) -> Result<Vec<Value>, ParseError> {
        let array = json
            .as_array()
            .ok_or_else(|| ParseError::InvalidTypeError {
                json: json.to_string(),
                json_type: "Vec<Value>".to_string(),
            })?;
        Ok(array.to_owned())
    }
}
