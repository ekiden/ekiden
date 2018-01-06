use std::sync::Arc;

pub struct StorageServer {
  latest: Option<Arc<Vec<u8>>>,
}

impl StorageServer {
  pub fn new() -> Self {
    StorageServer {
      latest: None
    }
  }

  pub fn check_tx(tx: &[u8]) -> Result<(), String> {
    // @todo - check attestations
    Ok(())
  }

  pub fn get_latest(&self) -> Option<Arc<Vec<u8>>> {
    match self.latest {
      Some(ref val) => Some(Arc::clone(&val)),
      _ => None,
    }
  }

  pub fn set_latest(&mut self, latest: Vec<u8>) {
    self.latest = Some(Arc::new(latest));
  }

}

