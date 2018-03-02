use std::io::{Cursor, Write};
use std::slice::{from_raw_parts, from_raw_parts_mut};

use sgx_trts::trts::rsgx_raw_is_outside_enclave;

use protobuf::{self, Message, MessageStatic};

use ekiden_common::error::Result;

/// Type which may be written into a `Write`.
pub trait Writable {
    /// Write the contents of self into given writer.
    ///
    /// Returns the number of bytes written.
    fn write_to(&self, writer: &mut Write) -> Result<usize>;
}

impl<M: Message> Writable for M {
    fn write_to(&self, writer: &mut Write) -> Result<usize> {
        self.write_to_writer(writer)?;

        Ok(self.compute_size() as usize)
    }
}

/// Deserialize request buffer from untrusted memory.
///
/// # EDL
///
/// In order for this function to work, the source buffer must be declared using
/// the [user_check] attribute in the EDL.
///
/// # Panics
///
/// This function will panic if the source buffer is not in untrusted memory as
/// this may compromise enclave security. Failing to deserialize the request
/// buffer will also cause a panic.
pub fn read_enclave_request<D>(src: *const u8, src_length: usize) -> D
where
    D: Message + MessageStatic,
{
    // Ensure that request data is in untrusted memory. This is required because
    // we are using user_check in the EDL so we must do all checks manually. If
    // the pointer was inside the enclave, we could expose arbitrary parts of
    // enclave memory.
    if !rsgx_raw_is_outside_enclave(src, src_length) {
        panic!("Security violation: source buffer must be in untrusted memory");
    }

    let src = unsafe { from_raw_parts(src, src_length) };
    protobuf::parse_from_bytes(src).expect("Malformed enclave request")
}

/// Copy enclave buffer in trusted memory to response buffer in untrusted memory.
///
/// # EDL
///
/// In order for this function to work, the destination buffer must be declared
/// using the [user_check] attribute in the EDL.
///
/// # Panics
///
/// This function will panic if the destination buffer is too small to hold the
/// content of the source buffer or if the destination buffer is not in untrusted
/// memory as this may compromise enclave security.
pub fn write_enclave_response<S>(src: &S, dst: *mut u8, dst_capacity: usize, dst_length: *mut usize)
where
    S: Writable,
{
    // Ensure that response data is in untrusted memory. This is required because
    // we are using user_check in the EDL so we must do all checks manually. If
    // the pointer was inside the enclave, we could overwrite arbitrary parts of
    // enclave memory.
    if !rsgx_raw_is_outside_enclave(dst, dst_capacity) {
        panic!("Security violation: destination buffer must be in untrusted memory");
    }

    // Serialize message to output buffer.
    let dst = unsafe { from_raw_parts_mut(dst, dst_capacity) };
    let mut cursor = Cursor::new(dst);
    let length = src.write_to(&mut cursor)
        .expect("Failed to write enclave response");

    unsafe {
        *dst_length = length;
    }
}
