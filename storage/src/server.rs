
pub struct StorageServer {
  latest: Option<Vec<u8>>
}

impl StorageServer {
  pub fn new() -> Self {
    StorageServer {
      latest: None
    }
  }
}

