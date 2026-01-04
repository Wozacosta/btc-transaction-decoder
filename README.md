# Helpful Links:
Mastering Bitcoin, Chapter 6: https://github.com/bitcoinbook/bitcoinbook/blob/develop/ch06_transactions.adoc
Raw transaction format: https://developer.bitcoin.org/glossary.html#term-Serialized-transaction
bdk: https://github.com/bitcoindevkit/bdk

# Raw Transaction Example:
Mempool.space: https://mempool.space/testnet/tx/3c1804567a336c3944e30b3c2593970bfcbf5b15a40f4fc6b626a360ee0507f2
Blockstream.info: https://blockstream.info/testnet/tx/3c1804567a336c3944e30b3c2593970bfcbf5b15a40f4fc6b626a360ee0507f2?expand


# Read version
// NOTE: we chose u32 as the version field in a transaction is represented as 4 bytes / 32 bits
// NOTE: 2 hex char = 1 byte
/*
* Bits, bytes and the dreaded little-endian: https://edil.com.br/blog/bits-bytes-and-the-dreaded-little-endian
learnmeabitcoin.com:
hexadecimal overview: https://learnmeabitcoin.com/technical/hexadecimal
little endian overview: https://learnmeabitcoin.com/technical/little-endian
NOTE: bitcoin protocol is little endian, the least significant bytes are sent first
NOTE: little endian = little end first
*/
