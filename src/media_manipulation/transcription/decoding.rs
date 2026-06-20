use anyhow::Context;
use ffmpeg_next::{codec, decoder, frame, ChannelLayout, Packet, Stream};
use ffmpeg_next::format::{sample, Sample};
use ffmpeg_next::software::resampling;
use crate::media_manipulation::transcription::SAMPLE_RATE;

pub struct SampleCollector {
	decoder: decoder::Audio,
	resampler: resampling::Context,
	
	samples: Vec<f32>,
}

impl SampleCollector {
	pub fn new(audio_stream: Stream) -> anyhow::Result<Self> {
		let mut decoder = codec::context::Context::from_parameters(audio_stream.parameters())?
			.decoder()
			.audio()?;
		
		decoder.set_packet_time_base(audio_stream.time_base());
		
		let resampler = ffmpeg_next::software::resampler(
			(decoder.format(), decoder.channel_layout(), decoder.rate()),
			(Sample::F32(sample::Type::Packed), ChannelLayout::MONO, SAMPLE_RATE)
		)?;
		
		Ok(Self {
			decoder,
			resampler,
			
			samples: Vec::new(),
		})
	}
	
	pub fn into_samples(self) -> Vec<f32> {
		self.samples
	}
	
	pub fn receive_input_packet(&mut self, in_packet: Packet) -> anyhow::Result<()> {
		self.decoder.send_packet(&in_packet).context("Sending packet")?;
		self.decode_frames()
	}
	
	pub fn send_eof(&mut self) -> anyhow::Result<()> {
		self.decoder.send_eof()?;
		self.decode_frames()?;
		
		Ok(())
	}
	
	fn decode_frames(&mut self) -> anyhow::Result<()> {
		let mut in_frame = frame::Audio::empty();
		
		while self.decoder.receive_frame(&mut in_frame).is_ok() {
			let mut out_frame = frame::Audio::empty();
			self.resampler.run(&in_frame, &mut out_frame)?;
			
			self.samples.extend(out_frame.plane(0));
		}
		
		Ok(())
	}
}
