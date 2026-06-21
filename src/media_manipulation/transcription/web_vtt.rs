#[derive(Debug, Clone)]
pub struct VTTCue {
	pub text: String,
	pub start_time: f32,
	pub end_time: f32,
}

pub fn cues_to_web_vtt(cues: &[VTTCue]) -> String {
	let mut text = String::new();
	text.push_str("WEBVTT\n");
	
	for cue in cues {
		text.push('\n');
		
		text.push_str(&make_timestamp(cue.start_time));
		text.push_str(" --> ");
		text.push_str(&make_timestamp(cue.end_time));
		text.push('\n');
		
		text.push_str(&cue.text);
		text.push('\n');
	}
	
	text
}

fn make_timestamp(time: f32) -> String {
	let hours = (time / 3600.0).floor() as u32;
	let minutes = (time % 3600.0 / 60.0).floor() as u32;
	let seconds = (time % 60.0).floor() as u32;
	let milliseconds = ((time % 1.0) * 1000.0).round() as u32 % 1000;
	
	format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, milliseconds)
}
