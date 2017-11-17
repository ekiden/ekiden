// Inspired by https://www.ethereum.org/token

use std::collections::HashMap;

struct ContractError {
  message: String
}

struct Address {
  value: String
}

impl Address {
  fn from(addr: String) -> Address {
    Address {
      value: addr
    }
  }

  fn as_str(&self) -> &str {
    self.value.as_str()
  }
}

impl ToString for Address {
  fn to_string(&self) -> String {
    self.value.clone()
  }
}

pub struct TokenContract {
  name: String,
  symbol: String,
  total_supply: u64,
  balance_of: HashMap<String, u64>,
}

impl TokenContract {
  fn new(
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
      Some(b) if *b < value => Err(ContractError { message: String::from("Insufficient `from` balance") }),
      Some(b) => Ok(*b),
      None => Err(ContractError { message: String::from("Nonexistent `from` account") }),
    }
  }

  fn get_to_balance(&self, addr: &Address) -> Result<u64, ContractError> {
    match self.balance_of.get(addr.as_str()) {
      Some(b) => Ok(*b),
      None => Ok(0),
    }
  }
  
  fn do_transfer(&mut self, from: &Address, to: &Address, value: u64) -> Result<(), ContractError>{
    let from_balance = self.get_from_balance(from, value)?;
    let to_balance = self.get_to_balance(to)?;
    if to_balance + value <= to_balance {
      return Err(ContractError { message: String::from("Transfer value too large, overflow `to` account") })
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
  fn transfer(&mut self, msg_sender: &Address, to: &Address, value: u64) -> Result<(), ContractError>{
    return self.do_transfer(msg_sender, to, value);
  }
  //fn burn(value: u64) -> Result<(), ContractError> {
  //}
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let a1 = Address::from(String::from("testaddr"));
    let c = TokenContract::new(&a1, 8, String::from("Ekiden Tokiden"), String::from("EKI"));


  }

}
