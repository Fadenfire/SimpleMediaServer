use std::fmt::Formatter;
use std::path::{Path, PathBuf};

use serde::{de, Deserializer};
use serde::de::Visitor;

pub fn add_extension(path: &Path, extension: impl AsRef<Path>) -> PathBuf {
	let mut path = path.to_owned();
	
	match path.extension() {
		Some(ext) => {
			let mut ext = ext.to_os_string();
			ext.push(".");
			ext.push(extension.as_ref());
			path.set_extension(ext)
		}
		None => path.set_extension(extension.as_ref()),
	};
	
	path
}

const POWER_UNITS: &[char] = &['k', 'M', 'G', 'T', 'P', 'E', 'Z', 'Y'];

pub fn abbreviate_number(num: u64) -> String {
	let power = num.ilog(1000);
	if power <= 0 { return num.to_string(); }
	
	let x = num / 1000u64.pow(power);
	let unit = POWER_UNITS.get((power - 1) as usize).unwrap_or(&'?');
	
	format!("{}{}", x, unit)
}

pub fn deserialize_suffixed_number<'de, D>(deserializer: D) -> Result<u64, D::Error>
where D: Deserializer<'de>
{
	struct SVisitor;
	
	impl<'de> Visitor<'de> for SVisitor {
		type Value = u64;
		
		fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
			formatter.write_str("an integer optionally suffixed with k, M, G, etc")
		}
		
		fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
		where E: de::Error
		{
			Ok(v)
		}
		
		fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
		where E: de::Error
		{
			convert_suffixed_number(v).ok_or_else(|| de::Error::custom("Invalid number"))
		}
	}
	
	deserializer.deserialize_str(SVisitor)
}

pub fn convert_suffixed_number(string: &str) -> Option<u64> {
	let string = string.replace('_', "");
	
	let Some(unit) = string.chars().last() else { return None };
	
	let exp = POWER_UNITS.iter()
		.position(|u| *u == unit)
		.map(|i| i + 1)
		.unwrap_or(0) as u32;
	
	let Some(num): Option<u64> = string.strip_suffix(POWER_UNITS)
		.unwrap_or(&string)
		.parse().ok()
	else { return None };
	
	let result = num * 1000u64.pow(exp);
	
	Some(result)
}

#[cfg(test)]
mod tests {
	use crate::utils::convert_suffixed_number;
	
	#[test]
	fn test_convert_suffixed_number() {
		assert_eq!(convert_suffixed_number("190"), Some(190));
		assert_eq!(convert_suffixed_number("19_0"), Some(190));
		assert_eq!(convert_suffixed_number("19_0___8"), Some(1908));
		
		assert_eq!(convert_suffixed_number("1k"), Some(1000));
		assert_eq!(convert_suffixed_number("18M"), Some(18_000_000));
		assert_eq!(convert_suffixed_number("97G"), Some(97_000_000_000));
		assert_eq!(convert_suffixed_number("18_6__M"), Some(186_000_000));
		
		assert_eq!(convert_suffixed_number(""), None);
		assert_eq!(convert_suffixed_number("_____"), None);
		assert_eq!(convert_suffixed_number("-100"), None);
		assert_eq!(convert_suffixed_number("M"), None);
		assert_eq!(convert_suffixed_number("____k"), None);
	}
}