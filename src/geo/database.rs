use super::{GeoBBox, GeoFile, GeoIndex};
use neon::types::Finalize;
use std::{error::Error, path::PathBuf, result::Result, time::Instant};

pub struct GeoDB {
	data: GeoFile,
	index: GeoIndex,
}
unsafe impl Send for GeoDB {}

impl GeoDB {
	pub fn open(filename: &PathBuf, memory_size: usize) -> Result<Self, Box<dyn Error>> {
		let mut filename_index = filename.clone();
		filename_index.set_extension("index");

		let mut data = GeoFile::load(filename)?;

		let index = if filename_index.exists() {
			GeoIndex::load(&filename_index)?
		} else {
			GeoIndex::create(&filename_index, &mut data)?
		};

		Ok(GeoDB { data, index })
	}

	pub fn find(
		&mut self, bbox: &GeoBBox, start_index: usize, max_count: usize,
	) -> Result<(Vec<String>, usize), Box<dyn Error>> {
		let data = &mut self.data;

		let start = Instant::now();
		let (entries, next_index) = self.index.collect_leaves(bbox, start_index, max_count);
		println!("A {:?}", start.elapsed());

		let start = Instant::now();
		let entries: Vec<String> = entries
			.iter()
			.map(|node| String::from_utf8(data.read_range(node.value1, node.value2).unwrap().to_vec()).unwrap())
			.collect();

		println!("B {:?}", start.elapsed());
		Ok((entries, next_index))
	}
}

impl Finalize for GeoDB {}
