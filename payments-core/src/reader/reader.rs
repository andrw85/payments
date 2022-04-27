use crate::account::Transaction;

use anyhow::{anyhow, Result};
use csv::{ReaderBuilder, Trim};
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Serialize, Deserialize, Debug)]
struct TransactionType {
    #[serde(alias = "type")]
    transaction_type: String,
    client: u16,
    tx: u32,
    amount: f64,
}
/// Load transactions from a stream of bytes in csv format
///
/// It receives an object that satisfies the io::Read trait. It can read the transactions
/// that must be presented in CSV format, and produces a `Vec<Transaction>`,
/// which can be then sent to the Account struct for further processing them.

pub fn load_csv_transactions(reader: impl io::Read) -> Result<Vec<Transaction>> {
    // let rdr = csv::Reader::from_reader(reader).trim(Trim::All);
    let rdr = ReaderBuilder::new().trim(Trim::All).from_reader(reader);
    let mut iter = rdr.into_deserialize();

    let mut res = Vec::new();
    while let Some(result) = iter.next() {
        let record: TransactionType = result?;
        let tx = match record.transaction_type.as_str() {
            "deposit" => Transaction::Deposit(record.client, record.tx, record.amount),
            "withdrawal" => Transaction::Withdrawal(record.client, record.tx, record.amount),
            "dispute" => Transaction::Dispute(record.client, record.tx, record.amount),
            "resolve" => Transaction::Resolve(record.client, record.tx, record.amount),
            "chargeback" => Transaction::Chargeback(record.client, record.tx, record.amount),
            _ => return Err(anyhow!("Not a valid transaction type")),
        };
        res.push(tx);
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reading_csv_records() {
        let input = "\
type,client,tx,amount
deposit,1,1,1.0
deposit,2,2,2.0
deposit,1,3,2.0
withdrawal,1,4,1.0
withdrawal,2,5,3.0"
            .as_bytes();
        let res = load_csv_transactions(input).expect("failed reading csv records");

        assert_eq!(res.len(), 5);
        assert_eq!(
            res,
            vec![
                Transaction::Deposit(1, 1, 1.0),
                Transaction::Deposit(2, 2, 2.0),
                Transaction::Deposit(1, 3, 2.0),
                Transaction::Withdrawal(1, 4, 1.0),
                Transaction::Withdrawal(2, 5, 3.0)
            ]
        )
    }
}
