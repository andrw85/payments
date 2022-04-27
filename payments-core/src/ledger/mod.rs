use super::account::{Account, ClientId};
use std::collections::HashMap;

/// A Ledger is the basic type that hold's a collection of user Accounts.
pub type Ledger = HashMap<ClientId, Account>;

/// Prints to stdout all of the accounts stored in the Ledger
pub fn print_ledger(ledger: Ledger) {
    println!("client, available, held, total, locked");
    for (key, value) in ledger {
        println!("{},{}", key, value.to_string());
    }
}
