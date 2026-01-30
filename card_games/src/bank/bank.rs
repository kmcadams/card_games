#[derive(Debug, Clone)]
pub struct Bank {
    balance: u32,
}

impl Bank {
    pub fn new(balance: u32) -> Self {
        Self { balance }
    }

    pub fn balance(&self) -> u32 {
        self.balance
    }

    pub fn withdraw(&mut self, amount: u32) -> bool {
        if self.balance >= amount {
            self.balance -= amount;
            true
        } else {
            false
        }
    }

    pub fn deposit(&mut self, amount: u32) {
        self.balance += amount;
    }
}
