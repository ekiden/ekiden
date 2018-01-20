pub struct State {
    latest: Option<Vec<u8>>,
}

impl State {
    pub fn new() -> Self {
        State { latest: None }
    }

    pub fn check_tx(_tx: &[u8]) -> Result<(), String> {
        // @todo - check attestations
        // @todo - check that this was based off latest
        Ok(())
    }

    pub fn get_latest(&self) -> Option<Vec<u8>> {
        match self.latest {
            // storage_grpc requires moving the Vec.
            // @todo replace with Arc?
            Some(ref val) => Some(val.clone()),
            _ => None,
        }
    }

    pub fn set_latest(&mut self, latest: Vec<u8>) {
        println!("new state: {}", String::from_utf8_lossy(&latest));
        self.latest = Some(latest);
    }
}
