mod account;

pub use account::{Account, Amount, ClientId, Transaction, Tx};

#[cfg(test)]
mod tests {
    use super::account::{Account, Transaction};
    #[test]
    fn test_create_transaction_for_account() {
        let tx = Transaction::Deposit(1, 1, 1.0);
        assert_eq!(Transaction::Deposit(1, 1, 1.0), tx);

        let mut account = Account::new(1);
        let res = account.process(tx);
        assert!(res.is_ok());
    }
}
