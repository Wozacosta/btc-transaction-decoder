use serde::{Serialize, Serializer};
use std::{fmt, io::Read};

#[derive(Debug, Serialize)]
struct Transaction {
    version: u64,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
}

#[derive(Debug, Serialize)]
struct Input {
    txid: String,
    output_index: u64, // TODO: change to u64
    script_sig: String,
    sequence: u64, // TODO: change to u64
}

#[derive(Debug, Serialize)]
struct Output {
    #[serde(serialize_with = "as_btc")]
    amount: Amount,
    script_pubkey: String,
}

fn as_btc<S: Serializer, T: BitcoinValue>(t: &T, s: S) -> Result<S::Ok, S::Error> {
    let btc = t.to_btc();
    s.serialize_f64(btc)
}

#[derive(Debug, Serialize)]
struct Amount(u64);

trait BitcoinValue {
    fn to_btc(&self) -> f64;
}

impl BitcoinValue for Amount {
    fn to_btc(&self) -> f64 {
        self.0 as f64 / 100_000_000.0
    }
}

fn read_nb_bytes(nb_bytes: usize, bytes: &mut &[u8]) -> u64 {
    let mut buffer = vec![0; nb_bytes];
    // let mut buffer = [0; nb_bytes];
    bytes.read(&mut buffer).unwrap();
    dbg!("HERE");
    println!("nb_bytes = {nb_bytes}, bytes = {:?}", bytes);
    println!("buffer here = {:?}", buffer);
    match nb_bytes {
        1 => buffer[0] as u64,
        2 => u16::from_le_bytes(buffer.try_into().unwrap()) as u64,
        4 => u32::from_le_bytes(buffer.try_into().unwrap()) as u64,
        8 => u64::from_le_bytes(buffer.try_into().unwrap()),
        _ => panic!("unsupported integer size"),
    }
}

// doc https://github.com/bitcoinbook/bitcoinbook/blob/develop/ch06_transactions.adoc#length-of-transaction-input-list
fn read_compact_size(mut transaction_bytes: &mut &[u8]) -> u64 {
    let mut compact_size = [0];
    transaction_bytes.read(&mut compact_size).unwrap();

    println!("now, transaction bytes = {:?}", transaction_bytes);

    match compact_size[0] {
        0..253 => compact_size[0] as u64,
        253 => read_nb_bytes(2, &mut transaction_bytes),
        254 => read_nb_bytes(4, &mut transaction_bytes),
        255 => read_nb_bytes(8, &mut transaction_bytes),
    }
}

fn read_txid(mut transaction_bytes: &mut &[u8]) -> [u8; 32] {
    let mut buffer = [0; 32];

    transaction_bytes.read(&mut buffer).unwrap();
    buffer.reverse();
    buffer
}

fn read_script(mut transaction_bytes: &mut &[u8]) -> Vec<u8> {
    let script_size = read_compact_size(&mut transaction_bytes) as usize;
    let mut buffer = vec![0_u8; script_size];
    transaction_bytes.read(&mut buffer).unwrap();
    buffer
}

fn main() {
    let transaction_hex = "010000000242d5c1d6f7308bbe95c0f6e1301dd73a8da77d2155b0773bc297ac47f9cd7380010000006a4730440220771361aae55e84496b9e7b06e0a53dd122a1425f85840af7a52b20fa329816070220221dd92132e82ef9c133cb1a106b64893892a11acf2cfa1adb7698dcdc02f01b0121030077be25dc482e7f4abad60115416881fe4ef98af33c924cd8b20ca4e57e8bd5feffffff75c87cc5f3150eefc1c04c0246e7e0b370e64b17d6226c44b333a6f4ca14b49c000000006b483045022100e0d85fece671d367c8d442a96230954cdda4b9cf95e9edc763616d05d93e944302202330d520408d909575c5f6976cc405b3042673b601f4f2140b2e4d447e671c47012103c43afccd37aae7107f5a43f5b7b223d034e7583b77c8cd1084d86895a7341abffeffffff02ebb10f00000000001976a9144ef88a0b04e3ad6d1888da4be260d6735e0d308488ac508c1e000000000017a91476c0c8f2fc403c5edaea365f6a284317b9cdf7258700000000";
    let transaction_bytes = hex::decode(transaction_hex).unwrap();
    let mut bytes_slice = transaction_bytes.as_slice();

    let version = read_nb_bytes(4, &mut bytes_slice);
    let input_count = read_compact_size(&mut bytes_slice);

    let mut inputs = vec![];

    for _ in 0..input_count {
        let txid = hex::encode(read_txid(&mut bytes_slice));
        let output_index = read_nb_bytes(4, &mut bytes_slice);
        let script_sig = hex::encode(read_script(&mut bytes_slice));
        let sequence = read_nb_bytes(4, &mut bytes_slice);

        inputs.push(Input {
            txid,
            output_index,
            script_sig,
            sequence,
        });
    }
    let output_count = read_compact_size(&mut bytes_slice);
    let mut outputs = vec![];

    for _ in 0..output_count {
        let amount = Amount(read_nb_bytes(8, &mut bytes_slice));
        let script_pubkey = hex::encode(read_script(&mut bytes_slice));
        outputs.push(Output {
            amount,
            script_pubkey,
        })
    }
    let transaction = Transaction {
        version,
        inputs,
        outputs,
    };

    println!(
        "Transaction: {}",
        serde_json::to_string_pretty(&transaction).unwrap()
    );
}

#[cfg(test)]
mod test {
    use super::read_compact_size;

    #[test]
    fn test_read_compact_size() {
        let mut bytes = [1_u8].as_slice();
        let count = read_compact_size(&mut bytes);
        assert_eq!(count, 1);

        let mut bytes = [253_u8, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes);
        // NOTE: not 1, as we're in little endian land
        assert_eq!(count, 256);

        let mut bytes = [254_u8, 0, 0, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes);
        assert_eq!(count, 256_u64.pow(3)); //256 * 256);

        let mut bytes = [255_u8, 0, 0, 0, 0, 0, 0, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes);
        assert_eq!(count, 256_u64.pow(7));

        // NOTE: from https://mempool.space/api/tx/52539a56b1eb890504b775171923430f0355eb836a57134ba598170a2f8980c1
        let hex = "fd204e";
        let decoded = hex::decode(hex).unwrap();
        let mut bytes = decoded.as_slice();
        let count = read_compact_size(&mut bytes);
        assert_eq!(count, 20_000);
    }
}
