use abci::application::Application;
use abci::types;

//#[derive(Copy, Clone)]
#[derive(Clone)]
pub struct Ekidenmint {
  name: String,
}

impl Ekidenmint {
  pub fn new() -> Ekidenmint {
    Ekidenmint{
      name: String::from("test"),
    }
  }
}

impl Application for Ekidenmint {
  fn info(&self, req: &types::RequestInfo) -> types::ResponseInfo {
    println!("info");
    types::ResponseInfo::new()
  }

  fn set_option(&self, req: &types::RequestSetOption) -> types::ResponseSetOption {
    println!("set_option");
    types::ResponseSetOption::new()
  }

  fn query(&self, p: &types::RequestQuery) -> types::ResponseQuery {
    println!("query");
    types::ResponseQuery::new()
  }

  fn check_tx(&self, p: &types::RequestCheckTx) -> types::ResponseCheckTx {
    // @todo
    println!("check_tx");
    types::ResponseCheckTx::new()
  }

  fn init_chain(&self, p: &types::RequestInitChain) -> types::ResponseInitChain {
    // Plugin support in https://github.com/tendermint/basecoin/blob/master/app/app.go
    //println!("init_chain");
    types::ResponseInitChain::new()
  }

  fn begin_block(&self, p: &types::RequestBeginBlock) -> types::ResponseBeginBlock {
    // Plugin support in https://github.com/tendermint/basecoin/blob/master/app/app.go
    //println!("begin_block");
    types::ResponseBeginBlock::new()
  }

  fn deliver_tx(&self, p: &types::RequestDeliverTx) -> types::ResponseDeliverTx {
    // @todo
    println!("deliver_tx");
    println!("{}", String::from_utf8_lossy(p.get_tx()));
    types::ResponseDeliverTx::new()

  }

  fn end_block(&self, p: &types::RequestEndBlock) -> types::ResponseEndBlock {
    // Plugin support in https://github.com/tendermint/basecoin/blob/master/app/app.go
    //println!("end_block");
    types::ResponseEndBlock::new()
  }

  fn commit(&self, p: &types::RequestCommit) -> types::ResponseCommit {
    // @todo
    println!("commit");
    types::ResponseCommit::new()
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
