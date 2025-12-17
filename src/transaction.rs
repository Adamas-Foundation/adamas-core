use crate::wallet::Wallet;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub signature: String,
}

impl Transaction {
    pub fn new(sender_wallet: &Wallet, receiver: String, amount: u64) -> Self {
        let sender_pub = sender_wallet.public_key.clone();
        let data_to_sign = format!("{}{}{}", sender_pub, receiver, amount);
        let signature = sender_wallet.sign(&data_to_sign);

        Transaction {
            sender: sender_pub,
            receiver,
            amount,
            signature,
        }
    }

    pub fn verify(&self) -> bool {
        let data_to_check = format!("{}{}{}", self.sender, self.receiver, self.amount);
        Wallet::verify(&data_to_check, &self.signature, &self.sender)
    }
}