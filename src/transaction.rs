use serde::{Serialize, Serializer};

#[derive(Debug, Serialize)]
pub struct Transaction {
    pub transaction_id: Txid,
    pub version: u64,
    pub lock_time: u64,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
}

#[derive(Debug, Serialize)]
pub struct Input {
    pub txid: Txid,
    pub output_index: u64,
    pub script_sig: String,
    pub sequence: u64,
}

#[derive(Debug, Serialize)]
pub struct Output {
    #[serde(serialize_with = "serialize_amount_as_btc")]
    pub amount: Amount,
    pub script_pubkey: String,
}

/// Serde helper: serialize satoshis as BTC (float) in JSON output.
///
/// Beginner note:
/// - `serialize_with` lets you control how a field is written without changing the type.
/// - Here, we keep Amount internal as satoshis but print BTC for readability.
fn serialize_amount_as_btc<S: Serializer>(amount: &Amount, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_f64(amount.as_btc())
}

/// Newtype wrapper around satoshis.
///
/// Beginner note:
/// - `Amount(u64)` prevents confusing "random u64" with "money".
/// - You can enforce invariants later (max money, etc).
#[derive(Debug, Copy, Clone)]
pub struct Amount(u64);

impl Amount {
    pub fn from_sat(satoshi: u64) -> Amount {
        Amount(satoshi)
    }

    pub fn as_sat(&self) -> u64 {
        self.0
    }

    pub fn as_btc(&self) -> f64 {
        self.0 as f64 / 100_000_000.0
    }
}

/// A typed txid wrapper.
///
/// Beginner note:
/// - Inside raw tx bytes, hashes are stored in an order different from explorer display.
/// - We keep raw bytes as-is, and reverse only for display/serialization.
#[derive(Debug, Copy, Clone)]
pub struct Txid([u8; 32]);

impl Txid {
    pub fn from_bytes(bytes: [u8; 32]) -> Txid {
        Txid(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Serialize txid as hex string in the common explorer format (reversed).
impl Serialize for Txid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut bytes = self.0;
        bytes.reverse();
        serializer.serialize_str(&hex::encode(bytes))
    }
}
