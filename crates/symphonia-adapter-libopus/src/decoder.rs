use std::ffi::c_int;

use log::{error, warn};
use symphonia_core::errors::{Error, Result};

pub enum ErrorCode {
    BadArg,
    BufferTooSmall,
    InternalError,
    InvalidPacket,
    Unimplemented,
    InvalidState,
    AllocFail,
}
impl ErrorCode {
    fn from_c_int(code: c_int) -> Option<Self> {
        use ErrorCode::*;
        match code {
            opusic_sys::OPUS_OK => None,
            opusic_sys::OPUS_BAD_ARG => Some(BadArg),
            opusic_sys::OPUS_BUFFER_TOO_SMALL => Some(BufferTooSmall),
            opusic_sys::OPUS_INTERNAL_ERROR => Some(InternalError),
            opusic_sys::OPUS_INVALID_PACKET => Some(InvalidPacket),
            opusic_sys::OPUS_UNIMPLEMENTED => Some(Unimplemented),
            opusic_sys::OPUS_INVALID_STATE => Some(InvalidState),
            opusic_sys::OPUS_ALLOC_FAIL => Some(AllocFail),
            _ => None,
        }
    }

    const fn as_str(&self) -> &str {
        use ErrorCode::*;
        match self {
            BadArg => "One or more invalid/out of range arguments.",
            BufferTooSmall => "The mode struct passed is invalid.",
            InternalError => "An internal error was detected.",
            InvalidPacket => "The compressed data passed is corrupted.",
            Unimplemented => "Invalid/unsupported request number.",
            InvalidState => "An encoder or decoder structure is invalid or already freed.",
            AllocFail => "Memory allocation has failed. ",
        }
    }
}

#[derive(Debug)]
pub(crate) struct Decoder {
    ptr: *mut opusic_sys::OpusDecoder,
    channels: u32,
}

impl Drop for Decoder {
    fn drop(&mut self) {
        unsafe {
            opusic_sys::opus_decoder_destroy(self.ptr);
        }
    }
}

unsafe impl Send for Decoder {}
unsafe impl Sync for Decoder {}

impl Decoder {
    pub(crate) fn new(sample_rate: u32, channels: u32) -> Result<Self> {
        let mut error = 0;
        let ptr = unsafe {
            opusic_sys::opus_decoder_create(sample_rate as i32, channels as c_int, &mut error)
        };
        if let Some(err) = ErrorCode::from_c_int(error) {
            let errstr = err.as_str();
            error!("decoder failed to create with error: {errstr}");
            return Err(Error::DecodeError("opus: error creating decoder: {errstr}"));
        }
        Ok(Self { ptr, channels })
    }

    pub(crate) fn decode(&mut self, input: &[u8], output: &mut [i16]) -> Result<usize> {
        let ptr = match input.len() {
            0 => std::ptr::null(),
            _ => input.as_ptr(),
        };
        let len = unsafe {
            opusic_sys::opus_decode(
                self.ptr,
                ptr,
                len(input),
                output.as_mut_ptr(),
                len(output) / self.channels as c_int,
                0 as c_int,
            )
        };
        if let Some(err) = ErrorCode::from_c_int(len) {
            let errstr = err.as_str();
            warn!("decode failed with error: {errstr}");
            return Err(Error::DecodeError("opus: decode failed: {errstr}"));
        }
        Ok(len as usize)
    }

    pub(crate) fn reset(&mut self) {
        let result =
            unsafe { opusic_sys::opus_decoder_ctl(self.ptr, opusic_sys::OPUS_RESET_STATE) };

        if let Some(err) = ErrorCode::from_c_int(result) {
            let errstr = err.as_str();
            warn!("reset failed with error {errstr}");
        }
    }
}

fn check_len(val: usize) -> c_int {
    match c_int::try_from(val) {
        Ok(val2) => val2,
        Err(_) => panic!("length out of range: {}", val),
    }
}

#[inline]
fn len<T>(slice: &[T]) -> c_int {
    check_len(slice.len())
}
