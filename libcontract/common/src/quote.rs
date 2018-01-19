pub const QUOTE_CONTEXT_LEN: usize = 8;
pub type QuoteContext = [u8; QUOTE_CONTEXT_LEN];
/// Secure channel contract -> client RA (EkQ-CoCl).
pub const QUOTE_CONTEXT_SC_CONTRACT_TO_CLIENT: QuoteContext = [69, 107, 81, 45, 67, 111, 67, 108];
/// Secure channel client -> contract RA (EkQ-ClCo).
pub const QUOTE_CONTEXT_SC_CLIENT_TO_CONTRACT: QuoteContext = [69, 107, 81, 45, 67, 108, 67, 111];
