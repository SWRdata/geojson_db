use super::{GeoBBox, GeoNode};
use geojson::Feature;
use std::{
	error::Error,
	fs::File,
	io::{BufRead, BufReader, Read, Seek, SeekFrom},
	path::PathBuf,
	result::Result,
	str::FromStr,
};

pub struct GeoFile {
	file: BufReader<File>,
}
impl GeoFile {
	pub fn load(filename: &PathBuf, max_memory: usize) -> Result<Self, Box<dyn Error>> {
		let file = BufReader::with_capacity(max_memory, File::open(filename).unwrap());
		Ok(Self { file })
	}

	pub fn read_range(&mut self, start: usize, length: usize) -> Result<Vec<u8>, Box<dyn Error>> {
		self.file.seek(SeekFrom::Start(start as u64))?;
		let mut buf = vec![0; length];
		self.file.read_exact(&mut buf)?;
		Ok(buf)
	}

	pub fn read_ranges(&mut self, leaves: Vec<&GeoNode>) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
		Ok(leaves
			.iter()
			.map(|l| self.read_range(l.value1, l.value2).unwrap())
			.collect())
	}

	pub fn get_entries(&mut self) -> Result<Vec<GeoNode>, Box<dyn Error>> {
		let mut entries: Vec<GeoNode> = Vec::new();
		let mut line = String::new();
		let mut byte_count: usize;

		loop {
			let current_pos = self.file.stream_position()?;
			match self.file.read_line(&mut line) {
				Ok(count) => {
					if count == 0 {
						break;
					}
					byte_count = count;
				}
				Err(err) => {
					println!("{:?}", err);
					panic!()
				}
			}

			let feature = Feature::from_str(&line)?;
			entries.push(GeoNode::new_leaf(
				GeoBBox::from_geometry(&feature.geometry.unwrap()),
				current_pos as usize,
				byte_count,
			));

			line.clear();
		}

		Ok(entries)
	}
}
