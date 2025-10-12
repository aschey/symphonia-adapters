use std::ffi::c_int;

use log::{error, warn};
use symphonia_core::errors::{Error, Result};

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
        if error != opusic_sys::OPUS_OK {
            error!("decoder failed to create with error code {error}");
            return Err(Error::DecodeError("opus: error creating decoder"));
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
        if len < 0 {
            warn!("decode failed with error code {len}");
            return Err(Error::DecodeError("opus: decode failed"));
        }
        Ok(len as usize)
    }

    pub(crate) fn reset(&mut self) {
        let result =
            unsafe { opusic_sys::opus_decoder_ctl(self.ptr, opusic_sys::OPUS_RESET_STATE) };

        if result != opusic_sys::OPUS_OK {
            warn!("reset failed with error code {result}");
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
