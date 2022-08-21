use crate::model::account::{PrivateKey, PublicKey};
use crate::model::transaction::data_transaction::DataTransaction;
use crate::model::transaction::TransactionData::Transfer;
use crate::model::transaction::TransferTransaction;
use crate::model::TransactionData::Data;
use crate::util::{sign, BinarySerializer};

pub struct TransactionInfo {
    id: String,
    signed_transaction: SignedTransaction,
    status: ApplicationStatus,
    height: u32,
}

impl TransactionInfo {
    pub fn new(
        id: String,
        signed_transaction: SignedTransaction,
        status: ApplicationStatus,
        height: u32,
    ) -> TransactionInfo {
        TransactionInfo {
            id,
            signed_transaction,
            status,
            height,
        }
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn status(&self) -> ApplicationStatus {
        self.status
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn signed_tx(&self) -> &SignedTransaction {
        &self.signed_transaction
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ApplicationStatus {
    Succeed,
    ScriptExecutionFailed,
    Unknown,
}

#[derive(Clone)]
pub struct Transaction {
    data: TransactionData,
    fee: Amount,
    timestamp: u64,
    public_key: PublicKey,
    tx_type: u8,
    version: u8,
    chain_id: u8,
}

#[derive(Clone)]
pub struct Amount {
    fee: u64,
    fee_asset_id: Option<String>,
}

impl Amount {
    pub fn new(fee: u64, fee_asset_id: Option<String>) -> Amount {
        Amount { fee, fee_asset_id }
    }

    pub fn fee(&self) -> u64 {
        self.fee
    }

    pub fn fee_asset_id(&self) -> Option<String> {
        self.fee_asset_id.clone()
    }
}

impl Transaction {
    pub fn new(
        data: TransactionData,
        fee: Amount,
        timestamp: u64,
        public_key: PublicKey,
        tx_type: u8,
        version: u8,
        chain_id: u8,
    ) -> Transaction {
        Transaction {
            data,
            fee,
            timestamp,
            public_key,
            tx_type,
            version,
            chain_id,
        }
    }

    pub fn data(&self) -> &TransactionData {
        &self.data
    }

    pub fn fee(&self) -> Amount {
        self.fee.clone()
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn tx_type(&self) -> u8 {
        self.tx_type
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn chain_id(&self) -> u8 {
        self.chain_id
    }

    pub fn sign(&self, private_key: &PrivateKey) -> SignedTransaction {
        sign(self, private_key)
    }

    pub fn bytes(&self) -> Vec<u8> {
        BinarySerializer::body_bytes(self)
    }
}

#[derive(Clone)]
pub enum TransactionData {
    Transfer(TransferTransaction),
    Data(DataTransaction),
    Issue(),
}

impl TransactionData {
    pub fn transfer_tx(&self) -> Result<&TransferTransaction, String> {
        match self {
            Transfer(tx) => Ok(tx),
            _ => Err("failed".into()),
        }
    }

    pub fn data_tx(&self) -> Result<&DataTransaction, String> {
        match self {
            Data(tx) => Ok(tx),
            _ => Err("failed".into()),
        }
    }
}

pub struct SignedTransaction {
    transaction: Transaction,
    proofs: Vec<Vec<u8>>,
}

impl SignedTransaction {
    pub fn new(transaction: Transaction, proofs: Vec<Vec<u8>>) -> SignedTransaction {
        SignedTransaction {
            transaction,
            proofs,
        }
    }

    pub fn tx(&self) -> &Transaction {
        &self.transaction
    }

    pub fn proofs(&self) -> Vec<Vec<u8>> {
        self.proofs.clone()
    }
}
