use super::{GeoBBox, GeoFile, GeoIndex};
use neon::types::Finalize;
use std::{error::Error, path::PathBuf, result::Result, time::Instant};

pub struct GeoDB {
	index: GeoIndex,
	table: GeoFile,
}
unsafe impl Send for GeoDB {}

impl GeoDB {
	pub fn open(filename: &PathBuf, max_memory: usize) -> Result<Self, Box<dyn Error>> {
		let mut filename_index = filename.clone();
		filename_index.set_extension("idx");

		let mut filename_table = filename.clone();
		filename_table.set_extension("dat");

		let index: GeoIndex = if filename_index.exists() && filename_table.exists() {
			println!("load index");
			GeoIndex::load(&filename_index)?
		} else {
			println!("load file temporary");
			let data = &mut GeoFile::load(filename, max_memory)?;

			println!("create index");
			GeoIndex::create(data, &filename_index, &filename_table)?
		};

		println!("load file");
		let table: GeoFile = GeoFile::load(&filename_table, max_memory)?;

		Ok(GeoDB { index, table })
	}

	pub fn query_bbox(
		&mut self, bbox: &GeoBBox, start_index: usize, max_count: usize,
	) -> Result<(Vec<Vec<u8>>, usize), Box<dyn Error>> {
		let start = Instant::now();
		let (leaves, next_index) = self.index.query_bbox(bbox, start_index, max_count);
		println!("A {:?}", start.elapsed());

		let start = Instant::now();
		let chunks: Vec<Vec<u8>> = self.table.read_ranges(leaves)?;
		println!("B {:?}", start.elapsed());

		Ok((chunks, next_index))
	}
}

impl Finalize for GeoDB {}
