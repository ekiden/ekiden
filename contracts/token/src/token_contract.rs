// Inspired by https://www.ethereum.org/token

use std::collections::HashMap;
use libcontract_trusted::common::address::Address;
use libcontract_trusted::common::contract_error::ContractError;

pub struct TokenContract {
  name: String,
  symbol: String,
  total_supply: u64,
  balance_of: HashMap<String, u64>,
}

impl TokenContract {
  pub fn new(
      msg_sender: &Address,
      initial_supply: u64, 
      token_name: String,
      token_symbol: String) -> TokenContract {
    let decimals = 18;
    let total_supply = initial_supply * 10 ^ decimals;
    TokenContract {
      name: token_name.clone(),
      symbol: token_symbol.clone(),
      total_supply: total_supply,
      balance_of: {
        let mut h = HashMap::new();
        h.insert(msg_sender.to_string(), total_supply);
        h
      },
    }
  }
  
  // PRIVATE METHODS
  fn get_from_balance(&self, addr: &Address, value: u64) -> Result<u64, ContractError> {
    match self.balance_of.get(addr.as_str()) {
      None => Err(ContractError::new(String::from("Nonexistent `from` account"))),
      Some(b) if *b < value => Err(ContractError::new(String::from("Insufficient `from` balance"))),
      Some(b) => Ok(*b),
    }
  }

  fn get_to_balance(&self, addr: &Address) -> Result<u64, ContractError> {
    match self.balance_of.get(addr.as_str()) {
      Some(b) => Ok(*b),
      None => Ok(0),
    }
  }
  
  fn do_transfer(&mut self, from: &Address, to: &Address, value: u64) -> Result<(), ContractError> {
    let from_balance = self.get_from_balance(from, value)?;
    let to_balance = self.get_to_balance(to)?;
    if to_balance + value <= to_balance {
      return Err(ContractError::new(String::from("Transfer value too large, overflow `to` account")));
    }

    // Set new balances
    let previous_balances = from_balance + to_balance;
    let from_balance = from_balance - value;
    let to_balance = to_balance + value;
    self.balance_of.insert(from.to_string(), from_balance);
    self.balance_of.insert(to.to_string(), to_balance);
    
    //Emit Transfer(_from, _to, _value) event;
    assert_eq!(
      previous_balances,
      (self.balance_of.get(from.as_str()).unwrap() + self.balance_of.get(to.as_str()).unwrap()),
      "new balance sum should equal old balance sum after transfer"
    );

    return Ok(())
  }

  // PUBLIC METHODS
  // - callable over RPC
  pub fn get_name(&self) -> Result<String, ContractError> {
    Ok(self.name.clone())
  }

  pub fn get_symbol(&self) -> Result<String, ContractError> {
    Ok(self.symbol.clone())
  }

  pub fn get_balance(&self, msg_sender: &Address) -> Result<u64, ContractError> {
    self.get_to_balance(msg_sender)
  }

  pub fn transfer(&mut self, msg_sender: &Address, to: &Address, value: u64) -> Result<(), ContractError> {
    self.do_transfer(msg_sender, to, value)
  }

  pub fn burn(&mut self, msg_sender: &Address, value: u64) -> Result<(), ContractError> {
    let from_balance = self.get_from_balance(msg_sender, value)?;
    self.balance_of.insert(msg_sender.to_string(), from_balance - value);
    self.total_supply -= value;
    // Emit Burn(msg_sender, value) event;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new_contract() {
    let name = "Ekiden Token";
    let symbol = "EKI";
    let a1 = Address::from(String::from("testaddr"));
    let c = TokenContract::new(&a1, 8, String::from(name), String::from(symbol));
    assert_eq!(name, c.get_name().unwrap(), "name should be set");
    assert_eq!(symbol, c.get_symbol().unwrap(), "symbol should be set");
    assert!(0 < c.total_supply, "total_supply should be set");
  }

  #[test]
  fn get_initial_balance() {
    let a1 = Address::from(String::from("testaddr"));
    let c = TokenContract::new(&a1, 8, String::from("Ekiden Tokiden"), String::from("EKI"));
    let b = c.get_balance(&a1).expect("testaddr should have tokens");
    assert_eq!(c.total_supply, b, "creator should get all the tokens");
  }

  // @todo - add more tests

}
