use super::{GeoBBox, GeoFile, GeoIndex};
use neon::types::Finalize;
use std::{error::Error, path::PathBuf, result::Result};

pub type IteratorResult<'a> = (Vec<&'a [u8]>, usize);

pub struct GeoDB {
	index: GeoIndex,
	table: GeoFile,
}
unsafe impl Send for GeoDB {}

impl GeoDB {
	pub fn open(filename: &PathBuf) -> Result<Self, Box<dyn Error>> {
		let mut filename_index = filename.clone();
		filename_index.set_extension("idx");

		let mut filename_table = filename.clone();
		filename_table.set_extension("dat");

		let index: GeoIndex = if filename_index.exists() && filename_table.exists() {
			GeoIndex::load(&filename_index)?
		} else {
			let data = &mut GeoFile::load(filename)?;
			GeoIndex::create(data, &filename_index, &filename_table)?
		};

		let table: GeoFile = GeoFile::load(&filename_table)?;
		Ok(GeoDB { index, table })
	}

	pub fn query_bbox(
		&self, bbox: &GeoBBox, start_index: usize, max_count: usize,
	) -> Result<IteratorResult, Box<dyn Error>> {
		let (leaves, next_index) = self.index.query_bbox(bbox, start_index, max_count);
		let chunks: Vec<&[u8]> = self.table.read_ranges(leaves);
		Ok((chunks, next_index))
	}
}

impl Finalize for GeoDB {}
