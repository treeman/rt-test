use anyhow::bail;
use anyhow::Result;
use rust_decimal::Decimal;
use serde::{Serialize, Serializer};
use std::collections::HashMap;

use crate::atm::{Transaction, TransactionVariant};

/// Tx amount, used to avoid mixing deposits/withdrawals.
#[derive(Debug, Clone)]
enum TxAmount {
    Deposit(Decimal),
    Withdrawal(Decimal),
}

/// A single transaction.
#[derive(Debug)]
struct Tx {
    id: u32,
    amount: TxAmount,
    disputed: bool,
}

impl Tx {
    fn new(id: u32, amount: TxAmount) -> Self {
        Tx {
            id,
            amount,
            disputed: false,
        }
    }
}

/// An individual client.
///
/// Since transactions are held by the client, they're not globally unique.
#[derive(Debug)]
pub struct Client {
    id: u16,
    available: Decimal,
    held: Decimal,
    locked: bool,
    txs: HashMap<u32, Tx>,
}

impl Client {
    pub fn new(id: u16) -> Self {
        Self {
            id,
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            locked: false,
            txs: HashMap::new(),
        }
    }

    /// The total amount of a client.
    /// Implemented as a method instead of a field to ensure that it's always equal to available + held.
    pub fn total(&self) -> Decimal {
        self.available + self.held
    }

    /// Execute a transaction and update client state.
    pub fn execute(&mut self, t: Transaction) -> Result<()> {
        match t.variant {
            TransactionVariant::Deposit { amount } => {
                self.deposit(t.tx, amount);
            }
            TransactionVariant::Withdrawal { amount } => {
                self.withdrawal(t.tx, amount);
            }
            TransactionVariant::Dispute => {
                self.dispute(t.tx);
            }
            TransactionVariant::Resolve => {
                self.resolve(t.tx);
            }
            TransactionVariant::Chargeback => {
                self.chargeback(t.tx);
            }
        }

        // If these sanity checks screw up, something very serious has gone wrong
        // and we should call the fire department.
        // A better solution might be to enforce this via an unsigned Decimal type.
        if self.available < Decimal::ZERO {
            bail!("Failed available non-zero sanity check {:#?}", self);
        }
        if self.held < Decimal::ZERO {
            bail!("Failed held non-zero sanity check {:#?}", self);
        }

        Ok(())
    }

    fn deposit(&mut self, tx: u32, amount: Decimal) {
        // Only consider the 4 decimal points
        let amount = amount.round_dp(4);
        self.available += amount;
        self.insert_tx(Tx::new(tx, TxAmount::Deposit(amount)));
    }

    fn withdrawal(&mut self, tx: u32, amount: Decimal) {
        // Only consider the 4 decimal points
        let amount = amount.round_dp(4);
        // A withdrawal without enough funds should be silently ignored.
        if amount <= self.available {
            self.available -= amount;
            self.insert_tx(Tx::new(tx, TxAmount::Withdrawal(amount)));
        }
    }

    fn dispute(&mut self, tx: u32) {
        // Silently ignore non-existent txs
        if let Some(tx) = self.get_tx(tx) {
            tx.disputed = true;

            match tx.amount.clone() {
                TxAmount::Deposit(amount) => {
                    self.available -= amount;
                    self.held += amount;
                }
                TxAmount::Withdrawal(amount) => {
                    self.held += amount;
                }
            }
        }
    }

    fn resolve(&mut self, tx: u32) {
        // Silently ignore non-existent txs or txs that aren't disputed
        if let Some(tx) = self.get_tx(tx) {
            if !tx.disputed {
                return;
            }
            tx.disputed = false;

            match tx.amount.clone() {
                TxAmount::Deposit(amount) => {
                    self.available += amount;
                    self.held -= amount;
                }
                TxAmount::Withdrawal(amount) => {
                    self.held -= amount;
                }
            }
        }
    }

    fn chargeback(&mut self, tx: u32) {
        // Silently ignore non-existent txs or txs that aren't disputed
        if let Some(tx) = self.get_tx(tx) {
            if !tx.disputed {
                return;
            }
            tx.disputed = false;

            match tx.amount.clone() {
                TxAmount::Deposit(amount) => {
                    self.held -= amount;
                }
                TxAmount::Withdrawal(amount) => {
                    self.available += amount;
                    self.held -= amount;
                }
            }
            self.locked = true;
        }
    }

    fn insert_tx(&mut self, tx: Tx) {
        self.txs.insert(tx.id, tx);
    }

    fn get_tx(&mut self, tx: u32) -> Option<&mut Tx> {
        self.txs.get_mut(&tx)
    }
}

// Serialize Client via the struct ClientOutput in order to support
// output from the method `total()`.
impl Serialize for Client {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ClientOutput::from(self).serialize(serializer)
    }
}

#[derive(Debug, Serialize)]
struct ClientOutput {
    client: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

impl From<&Client> for ClientOutput {
    fn from(client: &Client) -> Self {
        Self {
            client: client.id,
            available: client.available,
            held: client.held,
            total: client.total(),
            locked: client.locked,
        }
    }
}
