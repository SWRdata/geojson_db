use super::GeoNode;
use memmap2::Mmap;
use std::{error::Error, fs::File, path::PathBuf, result::Result};

pub struct GeoTable {
	file: File,
	mmap: Mmap,
}
impl GeoTable {
	pub fn load(filename: &PathBuf) -> Result<Self, Box<dyn Error>> {
		let file = File::open(filename).unwrap();
		let mmap = unsafe { Mmap::map(&file)? };
		Ok(Self { file, mmap })
	}

	pub fn read_ranges(&self, leaves: Vec<&GeoNode>) -> Vec<&[u8]> {
		leaves
			.iter()
			.map(|l| &self.mmap[l.value1..l.value1 + l.value2])
			.collect()
	}
}
