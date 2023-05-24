use super::{GeoBBox, GeoNode};

use memmap2::Mmap;
use std::{error::Error, fs::File, path::PathBuf, result::Result, str::from_utf8, time::Instant};

pub struct GeoFile {
	_file: File,
	mmap: Mmap,
}
impl GeoFile {
	pub fn load(filename: &PathBuf) -> Result<Self, Box<dyn Error>> {
		let file = File::open(filename).unwrap();
		let mmap = unsafe { Mmap::map(&file)? };
		Ok(Self { _file: file, mmap })
	}

	pub fn read_range(&self, start: usize, length: usize) -> &[u8] {
		&self.mmap[start..start + length]
	}

	pub fn get_entries(&mut self) -> Result<Vec<GeoNode>, Box<dyn Error>> {
		let mut entries: Vec<GeoNode> = Vec::new();
		let mut line_no: usize = 0;
		let file_size: f64 = self.mmap.len() as f64 / 100.;
		let start = Instant::now();
		let mut current_pos: usize = 0;

		for i in 0..self.mmap.len() {
			if self.mmap[i] == 10 {
				// new line break

				line_no += 1;

				if line_no % 1000000 == 0 {
					println!(
						"get_entries: {}, {:.1}%, {:.0}/s, {:.1}MB/s",
						line_no,
						current_pos as f64 / file_size,
						line_no as f64 / start.elapsed().as_secs_f64(),
						current_pos as f64 / 1048576. / start.elapsed().as_secs_f64()
					)
				}
				let line = from_utf8(&self.mmap[current_pos..i])?;
				entries.push(GeoNode::new_leaf(
					GeoBBox::from_geojson(line),
					current_pos,
					i - current_pos,
				));
				current_pos = i + 1;
			}
		}

		Ok(entries)
	}
}
