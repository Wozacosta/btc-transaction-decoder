# Bitcoin Transaction Decoder (Rust) — Notes & References

## Helpful References

**Core documentation**
- *Mastering Bitcoin*, Chapter 6 — Transactions  
  https://github.com/bitcoinbook/bitcoinbook/blob/develop/ch06_transactions.adoc
- Bitcoin Developer Glossary — Serialized transaction format  
  https://developer.bitcoin.org/glossary.html#term-serialized-transaction

**Libraries / implementations**
- BDK (Bitcoin Dev Kit)  
  https://github.com/bitcoindevkit/bdk
- https://github.com/rust-bitcoin/rust-bitcoin

---

## Raw Transaction Examples (Testnet)

These links are useful to inspect decoded fields side-by-side with raw hex:

- Mempool.space  
  https://mempool.space/testnet/tx/3c1804567a336c3944e30b3c2593970bfcbf5b15a40f4fc6b626a360ee0507f2
- Blockstream.info (expanded view)  
  https://blockstream.info/testnet/tx/3c1804567a336c3944e30b3c2593970bfcbf5b15a40f4fc6b626a360ee0507f2?expand

---

## Transaction Version Field

- The transaction `version` is **4 bytes (32 bits)**.
- In Rust, it is naturally represented as a `u32`.
- Each **2 hex characters = 1 byte** in the serialized transaction.

---

## Endianness Notes (Critical)

Bitcoin uses **little-endian encoding** for most integer fields.

- **Little-endian** means the *least significant byte is encoded first*
- Mnemonic: *“little end first”*
- This applies to fields such as:
  - version
  - locktime
  - txid (when serialized internally)

### Recommended Reading
- Bits, bytes, and endianness  
  https://edil.com.br/blog/bits-bytes-and-the-dreaded-little-endian
- Learn Me a Bitcoin:
  - Hexadecimal overview  
    https://learnmeabitcoin.com/technical/hexadecimal
  - Little-endian explained  
    https://learnmeabitcoin.com/technical/little-endian

> ⚠️ **Decoder pitfall**: values often appear “reversed” when compared to block explorers, which usually display fields in human-friendly big-endian form.
