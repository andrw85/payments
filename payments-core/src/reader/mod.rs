/*
 The reader module provides  is a low level abstraction that helps reading transactions from a source.
*/
pub mod reader;

#[cfg(test)]
mod tests {
    // use super::account::{Account, Transaction};
    // #[test]
    // fn test_create_transaction_for_account() {
    //     let mut tx = Transaction::Deposit(1, 1, 1.0);
    //     assert_eq!(Transaction::Deposit(1, 1, 1.0), tx);

    //     let mut account = Account::new(1);
    //     let res = account.process(tx);
    //     assert!(res.is_ok());
    // }
}
