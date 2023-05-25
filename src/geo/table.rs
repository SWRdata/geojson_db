use super::GeoNode;
use std::{error::Error, fs::read, path::PathBuf, result::Result};

#[derive(Debug)]
pub struct GeoTable {
	data: Vec<u8>,
}
impl GeoTable {
	pub fn load(filename: &PathBuf) -> Result<Self, Box<dyn Error>> {
		Ok(Self { data: read(filename)? })
	}

	pub fn read_ranges(&self, leaves: Vec<&GeoNode>) -> Vec<&[u8]> {
		leaves
			.iter()
			.map(|l| &self.data[l.value1..l.value1 + l.value2])
			.collect()
	}
}
