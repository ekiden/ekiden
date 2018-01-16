// Tendermint ABCI Application for Ekiden
// This is a short-lived facade object, so all state needs to be protected by Arc/Mutex
// For reference on how to use the ABCI
// https://github.com/tendermint/abci
// https://github.com/tendermint/basecoin/
use std::sync::{Arc, Mutex};
use abci::application::Application;
use abci::types;

use state::State;

//#[derive(Copy, Clone)]
#[derive(Clone)]
pub struct Ekidenmint {
  state: Arc<Mutex<State>>,
}

impl Ekidenmint {
  pub fn new(state: Arc<Mutex<State>>) -> Ekidenmint {
    Ekidenmint{
      state: state,
    }
  }
}

impl Application for Ekidenmint {
  fn info(&self, req: &types::RequestInfo) -> types::ResponseInfo {
    // @todo - supposed to return information about app state
    // https://github.com/tendermint/abci
    println!("info");
    types::ResponseInfo::new()
  }

  fn set_option(&self, req: &types::RequestSetOption) -> types::ResponseSetOption {
    // @todo - Set application options
    // https://github.com/tendermint/abci
    println!("set_option {}:{}", req.get_key(), req.get_value());
    types::ResponseSetOption::new()
  }

  fn query(&self, p: &types::RequestQuery) -> types::ResponseQuery {
    // @todo - handle query requests
    // https://github.com/tendermint/abci
    println!("query");
    types::ResponseQuery::new()
  }

  fn check_tx(&self, p: &types::RequestCheckTx) -> types::ResponseCheckTx {
    let mut resp = types::ResponseCheckTx::new();
    match State::check_tx(p.get_tx()) {
      Ok(_) => {
	resp.set_code(types::CodeType::OK);
      },
      Err(error) => {
	resp.set_code(types::CodeType::BaseInvalidInput);
	resp.set_log(error);
      },
    }
    return resp;
  }

  fn init_chain(&self, _p: &types::RequestInitChain) -> types::ResponseInitChain {
    // Plugin support in https://github.com/tendermint/basecoin/blob/master/app/app.go
    //println!("init_chain");
    types::ResponseInitChain::new()
  }

  fn begin_block(&self, _p: &types::RequestBeginBlock) -> types::ResponseBeginBlock {
    // Plugin support in https://github.com/tendermint/basecoin/blob/master/app/app.go
    //println!("begin_block");
    types::ResponseBeginBlock::new()
  }

  fn deliver_tx(&self, p: &types::RequestDeliverTx) -> types::ResponseDeliverTx {
    println!("deliver_tx");
    let mut resp = types::ResponseDeliverTx::new();
    let tx = p.get_tx();
    match State::check_tx(tx) {
      Ok(_) => {
	// Respond
	resp.set_code(types::CodeType::OK);
      	// Set the state
	let mut s = self.state.lock().unwrap();
	s.set_latest(tx.to_vec());
      },
      Err(error) => {
	resp.set_code(types::CodeType::BaseEncodingError);
	resp.set_log(error);
      },
    }
    return resp;
  }

  fn end_block(&self, _p: &types::RequestEndBlock) -> types::ResponseEndBlock {
    // Plugin support in https://github.com/tendermint/basecoin/blob/master/app/app.go
    //println!("end_block");
    types::ResponseEndBlock::new()
  }

  fn commit(&self, p: &types::RequestCommit) -> types::ResponseCommit {
    // RequestCommit is empty
    println!("commit");
    let mut resp = types::ResponseCommit::new();
    // @todo - respond with Merkle root hash of the application state in `data`
    //resp.set_code(types::CodeType::OK);
    //resp.set_data(String::from("test data").into_bytes());
    //resp.set_log(String::from("test log"));
    return resp;
  }

  fn echo(&self, p: &types::RequestEcho) -> types::ResponseEcho {
    let mut response = types::ResponseEcho::new();
    response.set_message(p.get_message().to_owned());
    return response;
  }

  fn flush(&self, p: &types::RequestFlush) -> types::ResponseFlush {
    // Appears to be unused in https://github.com/tendermint/basecoin/blob/master/app/app.go
    //println!("flush");
    types::ResponseFlush::new()
  }

}
