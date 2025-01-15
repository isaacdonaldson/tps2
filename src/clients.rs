use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Display};

use crate::decimal_to_string;

// This allows us to order and compare id's in addition to all the other derive traits
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClientId(u16);

impl Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u16> for ClientId {
    fn from(id: u16) -> Self {
        Self(id)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Client {
    pub id: ClientId,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Client {
    pub fn new(id: u16) -> Self {
        Self {
            id: ClientId::from(id),
            available: Decimal::from(0),
            held: Decimal::from(0),
            total: Decimal::from(0),
            locked: false,
        }
    }

    pub fn new_with_values(id: u16, available: Decimal, held: Decimal, total: Decimal) -> Self {
        Self {
            id: ClientId::from(id),
            available,
            held,
            total,
            locked: false,
        }
    }

    pub fn is_valid(&self) -> bool {
        let zero_val = Decimal::from(0);

        let available_amt = self.total - self.held;
        if self.available < zero_val || available_amt != self.available {
            return false;
        }

        let held_amt = self.total - self.available;
        if self.held < zero_val || held_amt != self.held {
            return false;
        }

        let total_amt = self.available + self.held;
        if self.total < zero_val || total_amt != self.total {
            return false;
        }

        true
    }
}

// This is a mapping of client id to client
#[derive(Debug, Default)]
pub struct ClientList(HashMap<ClientId, Client>);

impl ClientList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_client(&self, id: &ClientId) -> Option<&Client> {
        self.0.get(id)
    }

    pub fn get_client_mut(&mut self, id: &ClientId) -> Option<&mut Client> {
        self.0.get_mut(id)
    }

    pub fn get_or_create_client(&mut self, id: &ClientId) -> &mut Client {
        self.0.entry(*id).or_insert_with(|| Client::new(id.0))
    }
}

impl Display for ClientList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "client, available, held, total, locked")?;
        for client in self.0.values() {
            let Client {
                id,
                available,
                held,
                total,
                locked,
            } = client;

            // Here we can rely on the decimal_to_string function to handle the rounding
            // so there is a single place to change the precision and less hardcoded logic
            writeln!(
                f,
                "{}, {1}, {2}, {3}, {4}",
                id,
                decimal_to_string(*available),
                decimal_to_string(*held),
                decimal_to_string(*total),
                locked
            )?;
        }
        Ok(())
    }
}
