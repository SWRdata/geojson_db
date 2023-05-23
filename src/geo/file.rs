use super::{GeoBBox, GeoNode};
use geojson::Feature;
use std::{
	error::Error,
	fs::File,
	io::{BufRead, BufReader, Read, Seek, SeekFrom},
	path::PathBuf,
	result::Result,
	str::FromStr,
	time::Instant,
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
		let mut line_no: usize = 0;
		let file_size: f64 = self.file.get_ref().metadata().unwrap().len() as f64 / 100.;
		let start = Instant::now();

		loop {
			line_no += 1;

			let current_pos = self.file.stream_position()?;
			if line_no % 1000000 == 0 {
				println!(
					"get_entries: {}, {:.1}%, {:.0}/s, {:.1}MB/s",
					line_no,
					current_pos as f64 / file_size,
					line_no as f64 / start.elapsed().as_secs_f64(),
					(current_pos as f64 / 1048576.) / start.elapsed().as_secs_f64()
				)
			}

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
