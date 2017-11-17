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
  
  fn _transfer(&mut self, from: &Address, to: &Address, value: u64) -> Result<(), ContractError>{
    let from_balance = {
      match self.balance_of.get(from.as_str()) {
        Some(b) => {
          if *b < value {
            return Err(ContractError { message: String::from("Insufficient `from` balance") })
          }
          *b
        },
        None => return Err(ContractError { message: String::from("Nonexistent `from` account") }),
      }
    };
    let to_balance = {
      match self.balance_of.get(to.as_str()) {
        Some(b) => *b,
        None => 0,
      }
    };
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
    assert!((self.balance_of.get(from.as_str()).unwrap() + self.balance_of.get(to.as_str()).unwrap()) == previous_balances);

    return Ok(())
  }

  fn transfer(&mut self, msg_sender: &Address, to: &Address, value: u64) -> Result<(), ContractError> {
    return self._transfer(msg_sender, to, value);
  }
}
