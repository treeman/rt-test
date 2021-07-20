use anyhow::Result;
use csv::{ReaderBuilder, Trim, Writer};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::path::Path;

use crate::client::Client;

/// The different types of actions a client can take.
///
/// Encode the transaction amount in deposit/withdrawal,
/// since it doesn't exist for the other variants.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum TransactionVariant {
    Deposit { amount: Decimal },
    Withdrawal { amount: Decimal },
    Dispute,
    Resolve,
    Chargeback,
}

/// A transaction describing an action a client can take.
#[derive(Debug, Deserialize)]
pub struct Transaction {
    pub client: u16,
    pub tx: u32,
    // Use internally tagged unions and struct flattening to encode common fields
    // next to type specific fields.
    // See: https://stackoverflow.com/questions/61205057/serde-internally-tagged-enum-with-common-fields
    #[serde(flatten)]
    pub variant: TransactionVariant,
}

/// An atm holding the state of the payment processor.
#[derive(Debug)]
pub struct Atm {
    pub clients: HashMap<u16, Client>,
}

impl Atm {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    /// Create a new atm and process transactions from the csv file specifeid by 'path'.
    pub fn from_path(path: &Path) -> Result<Self> {
        let mut atm = Atm::new();
        let mut reader = ReaderBuilder::new().trim(Trim::All).from_path(path)?;
        for row in reader.deserialize() {
            let transaction: Transaction = row?;
            atm.execute(transaction)?;
        }
        Ok(atm)
    }

    fn execute(&mut self, t: Transaction) -> Result<()> {
        let client = self.get_or_create_client(t.client);
        client.execute(t)
    }

    fn get_or_create_client(&mut self, client: u16) -> &mut Client {
        self.clients
            .entry(client)
            .or_insert_with(|| Client::new(client))
    }

    /// Print the state of clients in an csv format to stdout.
    pub fn print_csv(&self) -> Result<()> {
        let mut writer = Writer::from_writer(io::stdout());
        serialize(self, &mut writer)
    }

    /// Write the state of clients in an csv format to a string.
    #[allow(dead_code)]
    pub fn to_csv_string(&self) -> Result<String> {
        let mut writer = Writer::from_writer(vec![]);
        serialize(self, &mut writer)?;
        Ok(String::from_utf8(writer.into_inner()?)?)
    }
}

fn serialize<W: io::Write>(atm: &Atm, writer: &mut Writer<W>) -> Result<()> {
    for client in atm.clients.values() {
        writer.serialize(client)?;
    }
    writer.flush()?;
    Ok(())
}
