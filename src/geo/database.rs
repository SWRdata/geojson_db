use super::{file::GeoFileOptions, GeoBBox, GeoFile, GeoIndex, GeoTable};
use neon::types::Finalize;
use std::{error::Error, path::PathBuf, result::Result};

pub type IteratorResult<'a> = (Vec<&'a [u8]>, usize);

#[derive(Debug)]
pub struct GeoDB {
	index: GeoIndex,
	table: GeoTable,
}
unsafe impl Send for GeoDB {}
impl Finalize for GeoDB {}

impl GeoDB {
	pub fn open(filename: &PathBuf, opt: GeoFileOptions) -> Result<Self, Box<dyn Error>> {
		let stem = filename.file_name().unwrap().to_str().unwrap();
		let filename_index = filename.with_file_name(format!("{}.idx", stem));
		let filename_table = filename.with_file_name(format!("{}.dat", stem));

		let index: GeoIndex = if filename_index.exists() && filename_table.exists() {
			GeoIndex::load(&filename_index)?
		} else {
			GeoIndex::create(&mut GeoFile::load(filename, opt)?, &filename_index, &filename_table)?
		};

		Ok(GeoDB {
			index,
			table: GeoTable::load(&filename_table)?,
		})
	}

	pub fn query_bbox(
		&self, bbox: &GeoBBox, start_index: usize, max_count: usize,
	) -> Result<IteratorResult, Box<dyn Error>> {
		let (leaves, next_index) = self.index.query_bbox(bbox, start_index, max_count);
		let chunks: Vec<&[u8]> = self.table.read_ranges(leaves);
		Ok((chunks, next_index))
	}
}
