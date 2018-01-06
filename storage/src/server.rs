
pub struct StorageServer {
  latest: Option<Vec<u8>>
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
}

