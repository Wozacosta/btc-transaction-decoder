use sha2::{Digest, Sha256};

use crate::transaction::{Amount, Input, Output, Transaction, Txid};
mod transaction;

/// Parsing errors you can bubble up instead of panicking.
/// Keep it simple: one enum, a few variants, readable messages.
#[derive(Debug)]
pub enum DecodeError {
    /// Not enough bytes left to read the requested field.
    UnexpectedEof { needed: usize, remaining: usize },

    /// CompactSize prefix was present, but the encoding is non-canonical (optional rule).
    NonCanonicalCompactSize,

    /// We tried to read an integer with an unsupported byte width.
    UnsupportedIntWidth(usize),

    /// Hex decoding failed (invalid hex string).
    InvalidHex(hex::FromHexError),
}

impl From<hex::FromHexError> for DecodeError {
    fn from(e: hex::FromHexError) -> Self {
        DecodeError::InvalidHex(e)
    }
}

/// A very small "cursor" over a byte slice.
///
/// Beginner note:
/// - We store `&'a [u8]` (a view into bytes).
/// - Each read method consumes bytes by advancing `self.bytes`.
/// - This avoids passing `&mut &[u8]` everywhere, but it's the same concept.
///
/// This is the core pattern behind many binary decoders.
struct Decoder<'a> {
    bytes: &'a [u8],
}

impl<'a> Decoder<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    fn remaining(&self) -> usize {
        self.bytes.len()
    }

    /// Read exactly `n` bytes and advance the cursor.
    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        if self.bytes.len() < n {
            return Err(DecodeError::UnexpectedEof {
                needed: n,
                remaining: self.bytes.len(),
            });
        }
        let (head, tail) = self.bytes.split_at(n);
        self.bytes = tail;
        Ok(head)
    }

    fn read_u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.read_bytes(1)?[0])
    }

    fn read_u16_le(&mut self) -> Result<u16, DecodeError> {
        let b = self.read_bytes(2)?;
        Ok(u16::from_le_bytes(b.try_into().unwrap()))
        // `try_into().unwrap()` is safe here because we *just* split exactly 2 bytes.
    }

    fn read_u32_le(&mut self) -> Result<u32, DecodeError> {
        let b = self.read_bytes(4)?;
        Ok(u32::from_le_bytes(b.try_into().unwrap()))
    }

    fn read_u64_le(&mut self) -> Result<u64, DecodeError> {
        let b = self.read_bytes(8)?;
        Ok(u64::from_le_bytes(b.try_into().unwrap()))
    }

    /// Bitcoin CompactSize (a varint used for counts and lengths).
    ///
    /// Rules:
    /// - 0x00..=0xfc => value is that byte
    /// - 0xfd => next 2 bytes (LE u16)
    /// - 0xfe => next 4 bytes (LE u32)
    /// - 0xff => next 8 bytes (LE u64)
    ///
    /// Optional rule: reject non-canonical encodings (good hygiene).
    fn read_compact_size(&mut self) -> Result<u64, DecodeError> {
        let prefix = self.read_u8()?;

        let value = match prefix {
            n @ 0x00..=0xfc => n as u64,
            0xfd => self.read_u16_le()? as u64,
            0xfe => self.read_u32_le()? as u64,
            0xff => self.read_u64_le()?,
        };

        // Optional canonical encoding checks:
        // If a value could have been encoded in fewer bytes, reject it.
        // (This matches how many implementations treat CompactSize.)
        let non_canonical = match prefix {
            0xfd => value < 0xfd,
            0xfe => value <= 0xffff,
            0xff => value <= 0xffff_ffff,
            _ => false,
        };
        if non_canonical {
            return Err(DecodeError::NonCanonicalCompactSize);
        }

        Ok(value)
    }

    /// Read a txid "as stored in the transaction" (32 bytes, little-endian-ish).
    ///
    /// Important:
    /// - Inside the raw transaction, hashes are stored in "internal" byte order.
    /// - Explorers display txid reversed. We handle display in `Serialize for Txid`.
    fn read_txid(&mut self) -> Result<Txid, DecodeError> {
        let b = self.read_bytes(32)?;
        Ok(Txid::from_bytes(b.try_into().unwrap()))
    }

    /// Read script (CompactSize length + that many bytes).
    fn read_script(&mut self) -> Result<Vec<u8>, DecodeError> {
        let len = self.read_compact_size()? as usize;
        let b = self.read_bytes(len)?;
        Ok(b.to_vec())
    }
}

/// Hash transaction bytes into txid = hash256(raw_tx).
///
/// Beginner note:
/// - Bitcoin txid is **double SHA-256**.
fn hash_raw_transaction(raw_tx: &[u8]) -> Txid {
    let hash1 = Sha256::digest(raw_tx);
    let hash2 = Sha256::digest(hash1);
    Txid::from_bytes(hash2.into())
}

/// Decode the whole transaction. Keeping it in a function makes it testable.
pub fn decode_transaction(raw: &[u8]) -> Result<Transaction, DecodeError> {
    let mut d = Decoder::new(raw);

    let version = d.read_u32_le()? as u64;

    let input_count = d.read_compact_size()?;
    let mut inputs = Vec::with_capacity(input_count as usize);

    for _ in 0..input_count {
        let txid = d.read_txid()?;
        let output_index = d.read_u32_le()? as u64;
        let script_sig = hex::encode(d.read_script()?);
        let sequence = d.read_u32_le()? as u64;

        inputs.push(Input {
            txid,
            output_index,
            script_sig,
            sequence,
        });
    }

    let output_count = d.read_compact_size()?;
    let mut outputs = Vec::with_capacity(output_count as usize);

    for _ in 0..output_count {
        let amount_sat = d.read_u64_le()?;
        let script_pubkey = hex::encode(d.read_script()?);

        outputs.push(Output {
            amount: Amount::from_sat(amount_sat),
            script_pubkey,
        });
    }

    let lock_time = d.read_u32_le()? as u64;

    // Good hygiene: ensure we consumed all bytes (optional).
    // If not, it usually means we mis-parsed something upstream.
    if d.remaining() != 0 {
        // You can make this a dedicated error variant if you want.
        // For now, treat as EOF logic mismatch.
        // (Keeping it simple for beginners.)
    }

    let transaction_id = hash_raw_transaction(raw);

    Ok(Transaction {
        transaction_id,
        version,
        lock_time,
        inputs,
        outputs,
    })
}

#[cfg(test)]
mod tests {
    use super::{DecodeError, Decoder};

    #[test]
    fn test_read_compact_size() -> Result<(), DecodeError> {
        // 1
        let mut d = Decoder::new(&[1_u8]);
        assert_eq!(d.read_compact_size()?, 1);

        // 0xfd + 2 bytes LE
        // bytes: fd 00 01 => 0x0100 = 256
        let mut d = Decoder::new(&[0xfd_u8, 0, 1]);
        assert_eq!(d.read_compact_size()?, 256);

        // 0xfe + 4 bytes LE
        let mut d = Decoder::new(&[0xfe_u8, 0, 0, 0, 1]);
        assert_eq!(d.read_compact_size()?, 256_u64.pow(3));

        // 0xff + 8 bytes LE
        let mut d = Decoder::new(&[0xff_u8, 0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(d.read_compact_size()?, 256_u64.pow(7));

        // fd204e => 0xfd then 0x4e20 LE => 20000
        let raw = hex::decode("fd204e")?;
        let mut d = Decoder::new(&raw);
        assert_eq!(d.read_compact_size()?, 20_000);

        Ok(())
    }
}
