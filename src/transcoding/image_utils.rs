use anyhow::Context;
use ffmpeg_the_third::{decoder, format, frame};
use ffmpeg_the_third::format::Pixel;
use ffmpeg_the_third::software::scaling;
use image::flat::SampleLayout;
use image::FlatSamples;

pub fn extract_frame(
	demuxer: &mut format::context::Input,
	decoder: &mut decoder::Video,
	video_stream_index: usize
) -> anyhow::Result<Option<frame::Video>> {
	let mut frame = frame::Video::empty();
	let mut found_frame = false;
	
	for result in demuxer.packets() {
		let (stream, packet) = result?;
		
		if stream.index() == video_stream_index {
			decoder.send_packet(&packet).context("Decoding packet")?;
			
			if decoder.receive_frame(&mut frame).is_ok() {
				found_frame = true;
				break;
			}
		}
	}
	
	if found_frame {
		Ok(Some(frame))
	} else {
		Ok(None)
	}
}

pub fn scale_frame_rgb(
	cache: &mut Option<scaling::Context>,
	in_frame: &frame::Video,
	out_width: u32,
	out_height: u32
) -> anyhow::Result<frame::Video> {
	let context = match cache {
		Some(ctx) => {
			ctx.cached(
				in_frame.format(), in_frame.width(), in_frame.height(),
				Pixel::RGB24, out_width, out_height,
				scaling::Flags::BICUBIC,
			);
			
			ctx
		}
		None => {
			*cache = Some(scaling::Context::get(
				in_frame.format(), in_frame.width(), in_frame.height(),
				Pixel::RGB24, out_width, out_height,
				scaling::Flags::BICUBIC,
			).context("Creating image converter")?);
			
			cache.as_mut().unwrap()
		}
	};
	
	let mut out_frame = frame::Video::empty();
	context.run(in_frame, &mut out_frame).context("Converting frame")?;
	
	Ok(out_frame)
}

pub fn frame_image_sample_rgb(frame: &frame::Video) -> FlatSamples<&[u8]> {
	let components = frame.format().descriptor().unwrap().nb_components();
	
	let layout = SampleLayout {
		channels: components,
		channel_stride: 1,
		width: frame.width(),
		width_stride: components as usize,
		height: frame.height(),
		height_stride: frame.stride(0),
	};
	
	let samples = FlatSamples {
		samples: frame.data(0),
		layout,
		color_hint: None,
	};
	
	samples
}