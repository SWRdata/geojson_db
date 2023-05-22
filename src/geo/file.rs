use super::{GeoBBox, GeoNode};
use geojson::Feature;
use memmap2::Mmap;
use std::{
	error::Error,
	fs::File,
	path::PathBuf,
	result::Result,
	str::{from_utf8, FromStr},
};

pub struct GeoFile {
	mmap: Mmap,
}
impl GeoFile {
	pub fn load(filename: &PathBuf) -> Result<Self, Box<dyn Error>> {
		let file = File::open(filename).unwrap();
		let mmap = unsafe { memmap2::Mmap::map(&file)? };
		Ok(Self { mmap })
	}

	pub fn read_range(&mut self, start: usize, length: usize) -> Result<&[u8], Box<dyn Error>> {
		Ok(&self.mmap[start..start + length])
	}

	pub fn get_entries(&mut self) -> Result<Vec<GeoNode>, Box<dyn Error>> {
		let content = from_utf8(&self.mmap)?;

		let mut current_pos = 0;
		let mut entries: Vec<GeoNode> = Vec::new();
		for line in content.lines() {
			let end_pos = current_pos + line.len();

			let feature = Feature::from_str(line)?;
			entries.push(GeoNode::new_leaf(
				GeoBBox::from_geometry(&feature.geometry.unwrap()),
				current_pos,
				end_pos - current_pos,
			));

			current_pos = end_pos + 1; // +1 for the newline character
		}

		Ok(entries)
	}
}
