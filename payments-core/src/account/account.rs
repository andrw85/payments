// use clap::{Parser, Subcommand};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Tx = u32;
pub type ClientId = u16;
pub type Amount = f64;

/// A Transaction represents operations that the user can request to the payment system
///
/// There are 5 types of transactions that the system can handle:
/// - deposit: it's a credit to the client's asset account
/// - withdraw: it's a debit to the client's asset account,
/// - dispute: represents a client's claim that a transaction was erroneous and should be reversed.
/// - resolve: represents a resolution to a dispute, releasing the associated held funds.
/// - chargeback: it's the final state of a dispute and represents the client reversing a Deserialize, Debug, PartialEq, Clone)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Transaction {
    /// A deposit is a credit to the client's asset account
    Deposit(ClientId, Tx, Amount),
    /// A withdraw is a debit to the client's asset account,
    Withdrawal(ClientId, Tx, Amount),
    /// A dispute represents a client's claim that a transaction was erroneous and should be reversed.
    Dispute(ClientId, Tx, Amount),
    /// A resolve represents a resolution to a dispute, releasing the associated held funds.
    Resolve(ClientId, Tx, Amount),
    /// A chargeback is the final state of a dispute and represents the client reversing a transaction.
    Chargeback(ClientId, Tx, Amount),
}

/// An account belongs to a unique client and it used for tracking all of the user's transactions.
///
/// # Overview
///
/// An account can track all of the history of transactions and disputes that are currently active.
///
/// It uses its internal field called `records` for storing all transactions that have been already executed.
/// Transactions which are being disputed are removed from the `records` field and transfered to the `dispute` field.
/// Both `records` and `dispute` are implemented using HashMap<Tx,Transaction>, which ensures very fast lookups due
/// to the nature of the HashMap data structure.
///
/// It's worth noting that only `Transaction::Deposit` and `Transaction::Withdrawal` can be disputed. After a transaction is
/// disputed there are two possible solutions for the dispute:
/// - Transaction::Resolve: the dispute is cancelled and it won't take any effect, held funds are recovered.
/// - Transaction::Chargeback: the disputed is accepted and a previous deposit or withdrawal will be reversed.
///
///
#[derive(Debug)]
pub struct Account {
    client_id: ClientId,
    available: Amount,
    held: Amount,
    total: Amount,
    frozen: bool,
    records: HashMap<Tx, Transaction>,
    disputed: HashMap<Tx, Transaction>,
}

impl Account {
    /// Create an empty account by specifying a client id
    ///
    /// TODO: not checking if the client_id is valid
    pub fn new(client_id: ClientId) -> Account {
        Account {
            client_id: client_id,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            frozen: false,
            records: HashMap::new(),
            disputed: HashMap::new(),
        }
    }

    /// Evaluates and executes a Transaction.
    ///
    /// The transaction should have a valid client id matching the account's client id. Transactions cannot
    /// be executed if the account is frozen/locked.
    pub fn process(&mut self, tx: Transaction) -> Result<()> {
        match tx {
            Transaction::Deposit(client_id, _, _)
            | Transaction::Withdrawal(client_id, _, _)
            | Transaction::Dispute(client_id, _, _)
            | Transaction::Resolve(client_id, _, _)
            | Transaction::Chargeback(client_id, _, _) => {
                self.verify_transaction_valid(client_id)?
            }
        };

        match tx {
            Transaction::Deposit(_, tx, amount) => self.deposit(tx, amount)?,
            Transaction::Withdrawal(_, tx, amount) => self.withdrawal(tx, amount)?,
            Transaction::Dispute(_, tx, _) => self.dispute(tx)?,
            Transaction::Resolve(_, tx, _) => self.resolve(tx)?,
            Transaction::Chargeback(_, tx, _) => self.chargeback(tx)?,
        };

        match tx {
            Transaction::Deposit(_, txid, _) | Transaction::Withdrawal(_, txid, _) => {
                self.records.insert(txid, tx.clone());
            }
            _ => (),
        };

        Ok(())
    }

    fn verify_transaction_valid(&self, client_id: ClientId) -> Result<()> {
        if self.frozen {
            return Err(anyhow!("Transaction failed because account is frozen!"));
        }

        if client_id != self.client_id {
            return Err(anyhow!(
                "Transaction failed! not matching the account's client id."
            ));
        }
        Ok(())
    }

    fn deposit(&mut self, _tx: Tx, amount: Amount) -> Result<()> {
        self.available += amount;
        self.total += amount;
        Ok(())
    }

    fn withdrawal(&mut self, _tx: Tx, amount: Amount) -> Result<()> {
        if self.available < amount {
            return Err(anyhow!("withdrawal failed, insuficcient funds."));
        }
        if self.total < amount {
            return Err(anyhow!("withdrawal failed, insuficcient funds."));
        }
        self.available -= amount;
        self.total -= amount;
        Ok(())
    }

    fn dispute(&mut self, tx: Tx) -> Result<()> {
        if !self.records.contains_key(&tx) {
            return Err(anyhow!("dispute failed, not a valid transaction id."));
        }

        let disputed_transaction = &self.records[&tx];

        match disputed_transaction {
            Transaction::Deposit(_, _, amount) => {
                if self.available < *amount {
                    return Err(anyhow!("dispute failed, insuficcient funds."));
                }
                self.available -= amount;
                self.held += amount; // no need to update total since we move amout from available to held
            }
            Transaction::Withdrawal(_, _, amount) => {
                self.held += amount;
                self.total += amount; // we need to update the total, since this amount was not in available nor in held previously
            }
            _ => return Err(anyhow!("dispute failed, transaction referenced not valid.")),
        };

        self.disputed.insert(tx, disputed_transaction.clone());
        self.records.remove(&tx); // cannot dispute more than once the same transaction

        Ok(())
    }

    fn resolve(&mut self, tx: Tx) -> Result<()> {
        // resolve = cancel the dispute
        if !self.disputed.contains_key(&tx) {
            return Err(anyhow!("ignoring resolution, not a valid transaction id."));
        }

        let disputed_transaction = &self.disputed[&tx];
        match disputed_transaction {
            Transaction::Deposit(_, _, amount) => {
                // cancel the deposit dispute
                if self.held < *amount {
                    return Err(anyhow!(
                        "resolving dispute failed, insuficcient held funds."
                    ));
                }
                self.held -= amount;
                self.available += amount;
            }
            Transaction::Withdrawal(_, _, amount) => {
                // cancel the withdrawal dispute
                if self.held < *amount {
                    return Err(anyhow!(
                        "resolving dispute failed, insuficcient held funds."
                    ));
                }
                self.held -= amount;
                self.total -= amount;
            }
            _ => (), // never reached since disputed transactions are only deposits and withdrawals
        };

        self.disputed.remove(&tx);
        Ok(())
    }

    fn chargeback(&mut self, tx: Tx) -> Result<()> {
        // dispute was successful, apply charge
        if !self.disputed.contains_key(&tx) {
            return Err(anyhow!("ignoring chargeback, not a valid transaction id."));
        }

        let disputed_transaction = &self.disputed[&tx];
        match disputed_transaction {
            Transaction::Deposit(_, _, amount) | Transaction::Withdrawal(_, _, amount) => {
                if self.held < *amount {
                    return Err(anyhow!("chargeback failed, insuficcient held funds."));
                }
                if self.total < *amount {
                    return Err(anyhow!("chargeback failed, insuficcient total funds."));
                }
                self.held -= amount;
                self.total -= amount;
                self.frozen = true; // transactions might be fraudulatent threfore account is frozen.
            }
            _ => (), // never reached since disputed transactions are only deposits and withdrawals
        };
        self.disputed.remove(&tx);
        Ok(())
    }
}

impl ToString for Account {
    fn to_string(&self) -> String {
        format!(
            "{},{},{},{}",
            self.available, self.held, self.total, self.frozen,
        )
    }
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.client_id == other.client_id
            && self.available == other.available
            && self.held == other.held
            && self.total == other.total
            && self.frozen == other.frozen
            && self.records == other.records
            && self.disputed == other.disputed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_not_matching_accounts_client_id() {
        let mut account = Account::new(12);
        let tx = Transaction::Deposit(4, 1, 1.0); // deposit amount 1.0 for client 12, with tx(Transaction Id) 1
        let res = account.process(tx);

        assert_eq!(
            res.err().unwrap().to_string(),
            "Transaction failed! not matching the account's client id."
        );
        assert_eq!(
            account,
            Account {
                client_id: 12,
                available: 0.0,
                held: 0.0,
                total: 0.0,
                frozen: false,
                records: HashMap::new(), // no transaction recorded
                disputed: HashMap::new(),
            }
        );
    }

    #[test]
    fn test_deposit() {
        let mut account = Account::new(12);
        let tx = Transaction::Deposit(12, 1, 1.0); // deposit amount 1.0 for client 12, with tx(Transaction Id) 1
        let res = account.process(tx);

        assert!(res.is_ok());

        assert_eq!(
            account,
            Account {
                client_id: 12,
                available: 1.0,
                held: 0.0,
                total: 1.0,
                frozen: false,
                records: HashMap::from([(1, Transaction::Deposit(12, 1, 1.0))]), // 1 transaction
                disputed: HashMap::new(),
            }
        );
    }

    #[test]
    fn test_withdrawal_of_more_funds_than_available() {
        // initialize an account with available funds to 1.0
        let mut account = Account {
            client_id: 12,
            available: 1.0,
            held: 0.0,
            total: 1.0,
            frozen: false,
            records: HashMap::from([(1, Transaction::Deposit(12, 1, 1.0))]), // 1 transaction
            disputed: HashMap::new(),
        };

        let tx = Transaction::Withdrawal(12, 2, 3.0); // withdawal amount 1.0 for client 12, with tx(Transaction Id) 2
        let res = account.process(tx);
        assert_eq!(
            res.err().unwrap().to_string(),
            "withdrawal failed, insuficcient funds."
        );
    }
    #[test]
    fn test_not_valid_transaction_id_in_dispute() {
        let mut account = Account {
            client_id: 12,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            frozen: false,
            records: HashMap::from([
                (1, Transaction::Deposit(12, 1, 1.0)), // 2 transactions recorded
                (2, Transaction::Withdrawal(12, 2, 1.0)),
            ]),
            disputed: HashMap::new(),
        };

        let tx = Transaction::Dispute(12, 3, 1.0); // withdawal amount 1.0 for client 12, with tx(Transaction Id) 3 does not exist
        let res = account.process(tx);

        assert_eq!(
            res.err().unwrap().to_string(),
            "dispute failed, not a valid transaction id."
        );
    }
    #[test]
    fn test_disputing_a_withdrawal_of_accounts_total_funds() {
        let mut account = Account {
            client_id: 12,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            frozen: false,
            records: HashMap::from([
                (1, Transaction::Deposit(12, 1, 1.0)), // 2 transactions recorded
                (2, Transaction::Withdrawal(12, 2, 1.0)),
            ]),
            disputed: HashMap::new(),
        };

        let tx = Transaction::Dispute(12, 2, 1.0);
        let res = account.process(tx);

        assert!(res.is_ok());
        assert_eq!(
            account,
            Account {
                client_id: 12,
                available: 0.0,
                held: 1.0,
                total: 1.0,
                frozen: false,
                records: HashMap::from([
                    (1, Transaction::Deposit(12, 1, 1.0)), // 1 transactions recorded(the other one is being disputed)
                ]),
                disputed: HashMap::from([(2, Transaction::Withdrawal(12, 2, 1.0))]), //disputed transaction
            }
        );
    }
    #[test]
    fn test_dispute_partial_funds_withdrawal() {
        let mut account = Account {
            client_id: 12,
            available: 1.0,
            held: 0.0,
            total: 1.0,
            frozen: false,
            records: HashMap::from([
                (1, Transaction::Deposit(12, 1, 2.0)), // 2 transactions recorded
                (2, Transaction::Withdrawal(12, 2, 1.0)),
            ]),
            disputed: HashMap::new(),
        };

        let tx = Transaction::Dispute(12, 2, 1.0);
        let res = account.process(tx);

        assert!(res.is_ok());
        assert_eq!(
            account,
            Account {
                client_id: 12,
                available: 1.0,
                held: 1.0,
                total: 2.0,
                frozen: false,
                records: HashMap::from([
                    (1, Transaction::Deposit(12, 1, 2.0)), // 1 transactions recorded(the other one is being disputed)
                ]),
                disputed: HashMap::from([(2, Transaction::Withdrawal(12, 2, 1.0))]), //disputed transaction
            }
        );
    }
    #[test]
    fn test_disputing_a_deposit_after_no_funds_in_account() {
        let mut account = Account {
            client_id: 12,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            frozen: false,
            records: HashMap::from([
                (1, Transaction::Deposit(12, 1, 1.0)), // 2 transactions recorded
                (2, Transaction::Withdrawal(12, 2, 1.0)),
            ]),
            disputed: HashMap::new(),
        };
        let tx = Transaction::Dispute(12, 1, 0.0); // dispute deposit
        let res = account.process(tx);

        // this dispute should fail because after the withdrawal of all funds
        // we don't have any left in our account
        assert_eq!(
            res.err().unwrap().to_string(),
            "dispute failed, insuficcient funds."
        );
    }
    #[test]
    fn test_resolve_a_deposit_dispute() {
        let mut account = Account {
            client_id: 12,
            available: 0.0,
            held: 1.0,
            total: 1.0,
            frozen: false,
            records: HashMap::from([(1, Transaction::Deposit(12, 1, 1.0))]),
            disputed: HashMap::from([(1, Transaction::Deposit(12, 1, 1.0))]),
        };
        let tx = Transaction::Resolve(12, 1, 0.0); // resolve dispute
        let res = account.process(tx);

        assert!(res.is_ok());
        assert_eq!(
            account,
            Account {
                client_id: 12,
                available: 1.0,
                held: 0.0,
                total: 1.0,
                frozen: false,
                records: HashMap::from([(1, Transaction::Deposit(12, 1, 1.0))]),
                disputed: HashMap::new(),
            }
        );
    }

    #[test]
    fn test_resolve_non_existent_dispute() {
        let mut account = Account {
            client_id: 12,
            available: 0.0,
            held: 1.0,
            total: 1.0,
            frozen: false,
            records: HashMap::from([(1, Transaction::Deposit(12, 1, 1.0))]),
            disputed: HashMap::new(),
        };
        let tx = Transaction::Resolve(12, 1, 0.0); // resolving a non existent dispute
        let res = account.process(tx);
        assert_eq!(
            res.err().unwrap().to_string(),
            "ignoring resolution, not a valid transaction id."
        );
    }
    // TODO: add test resolving a withdrawal transaction (happy flow)
    #[test]
    fn test_chargeback_deposit() {
        let mut account = Account {
            client_id: 12,
            available: 0.0,
            held: 1.0,
            total: 1.0,
            frozen: false,
            records: HashMap::from([(1, Transaction::Deposit(12, 1, 1.0))]),
            disputed: HashMap::from([(1, Transaction::Deposit(12, 1, 1.0))]),
        };
        let tx = Transaction::Chargeback(12, 1, 0.0); // chargeback dispute
        let res = account.process(tx);

        assert!(res.is_ok());
        assert_eq!(
            account,
            Account {
                client_id: 12,
                available: 0.0,
                held: 0.0,
                total: 0.0,
                frozen: true,
                records: HashMap::from([(1, Transaction::Deposit(12, 1, 1.0))]),
                disputed: HashMap::new(),
            }
        );
    }
    // TODO: add test chrageback a withdrawal transaction (happy flow)
}
