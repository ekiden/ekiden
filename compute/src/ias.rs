use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{Error, ErrorKind, Read};
use std::str::FromStr;

use base64;
use reqwest;

use ekiden_core::enclave::quote::AttestationReport;
use ekiden_core::hex_encoded_struct;

/// Intel IAS API URL.
const IAS_API_URL: &'static str = "https://test-as.sgx.trustedservices.intel.com";
/// Intel IAS report endpoint.
///
/// See [https://software.intel.com/sites/default/files/managed/7e/3b/ias-api-spec.pdf].
const IAS_ENDPOINT_REPORT: &'static str = "/attestation/sgx/v2/report";

// SPID.
hex_encoded_struct!(SPID, SPID_LEN, 16);

/// IAS configuration.
///
/// The `spid` is a valid SPID obtained from Intel, while `pkcs12_archive`
/// is the path to the PKCS#12 archive (certificate and private key), which
/// will be used to authenticate to IAS.
pub struct IASConfiguration {
    /// SPID assigned by Intel.
    pub spid: SPID,
    /// PKCS#12 archive containing the identity for authenticating to IAS.
    pub pkcs12_archive: String,
}

/// IAS (Intel Attestation Service) interface.
#[derive(Clone)]
pub struct IAS {
    /// SPID assigned by Intel.
    spid: SPID,
    /// Client used for IAS requests.
    client: Option<reqwest::Client>,
}

impl IAS {
    /// Construct new IAS interface.
    pub fn new(config: Option<IASConfiguration>) -> io::Result<IAS> {
        match config {
            Some(config) => {
                Ok(IAS {
                    spid: config.spid.clone(),
                    client: {
                        // Read and parse PKCS#12 archive.
                        let mut buffer = Vec::new();
                        File::open(&config.pkcs12_archive)?.read_to_end(&mut buffer)?;
                        let identity = match reqwest::Identity::from_pkcs12_der(&buffer, "") {
                            Ok(identity) => identity,
                            _ => {
                                return Err(Error::new(
                                    ErrorKind::Other,
                                    "Failed to load IAS credentials",
                                ))
                            }
                        };

                        // Create client with the identity.
                        match reqwest::ClientBuilder::new().identity(identity).build() {
                            Ok(client) => Some(client),
                            _ => {
                                return Err(Error::new(
                                    ErrorKind::Other,
                                    "Failed to create IAS client",
                                ))
                            }
                        }
                    },
                })
            }
            None => Ok(IAS {
                spid: SPID([0; SPID_LEN]),
                client: None,
            }),
        }
    }

    /// Make authenticated web request to IAS.
    fn make_request(
        &self,
        endpoint: &str,
        data: &HashMap<&str, String>,
    ) -> io::Result<reqwest::Response> {
        let endpoint = format!("{}{}", IAS_API_URL, endpoint);

        let client = match self.client {
            Some(ref client) => client,
            None => return Err(Error::new(ErrorKind::Other, "IAS is not configured")),
        };

        match client.post(&endpoint).json(&data).send() {
            Ok(response) => Ok(response),
            _ => return Err(Error::new(ErrorKind::Other, "Request to IAS failed")),
        }
    }

    /// Make authenticated web request to IAS report endpoint.
    pub fn verify_quote(&self, nonce: &[u8], quote: &[u8]) -> io::Result<AttestationReport> {
        // Generate mock report when client is not configured.
        if self.client.is_none() {
            let report = AttestationReport::new(
                // TODO: Generate other mock fields.
                format!(
                    "{{\"isvEnclaveQuoteBody\": \"{}\"}}",
                    base64::encode(&quote)
                ).into_bytes(),
                vec![],
                vec![],
            );

            return Ok(report);
        }

        let mut request = HashMap::new();
        request.insert("isvEnclaveQuote", base64::encode(&quote));
        request.insert("nonce", base64::encode(&nonce));

        let mut response = self.make_request(IAS_ENDPOINT_REPORT, &request)?;
        if !response.status().is_success() {
            return Err(Error::new(ErrorKind::Other, "Request to IAS failed"));
        }

        Ok(AttestationReport::new(
            // TODO: Handle errors.
            response.text().unwrap().into_bytes(),
            response
                .headers()
                .get_raw("X-IASReport-Signature")
                .unwrap()
                .one()
                .unwrap()
                .to_vec(),
            response
                .headers()
                .get_raw("X-IASReport-Signing-Certificate")
                .unwrap()
                .one()
                .unwrap()
                .to_vec(),
        ))
    }

    /// Get configured SPID.
    pub fn get_spid(&self) -> &[u8; SPID_LEN] {
        &self.spid.0
    }
}
