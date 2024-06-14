use std::ffi::c_int;

pub mod in_memory_muxer;
pub mod hardware_device;

pub fn av_error(code: c_int) -> Result<c_int, ffmpeg_next::Error> {
	match code {
		0.. => Ok(code),
		_ => Err(ffmpeg_next::Error::from(code)),
	}
}