
#[derive(Debug)]
pub struct Address {
    value: String
}

impl Address {
    pub fn from(addr: String) -> Address {
        Address {
            value: addr
        }
    }

    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

impl ToString for Address {
    fn to_string(&self) -> String {
        self.value.clone()
    }
}
