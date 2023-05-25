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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::geo::GeoBBox;
	use std::path::PathBuf;

	// Testing loading a file
	#[test]
	fn geo_table_read_ranges() -> Result<(), Box<dyn Error>> {
		let filename = PathBuf::from("testdata/points.csv.gz");
		let geo_table = GeoTable::load(&filename)?;
		assert_eq!(geo_table.data.len(), 402237);

		let leaf1 = GeoNode::new_leaf(GeoBBox::new_empty(), 0, 8);
		let leaf2 = GeoNode::new_leaf(GeoBBox::new_empty(), 100000, 8);
		let leaf3 = GeoNode::new_leaf(GeoBBox::new_empty(), geo_table.data.len() - 8, 8);
		let data1 = [31, 139, 8, 0, 0, 0, 0, 0];
		let data2 = [175, 169, 186, 191, 247, 148, 69, 15];
		let data3 = [217, 205, 171, 145, 237, 61, 26, 0];

		let ranges = geo_table.read_ranges(vec![&leaf1, &leaf2, &leaf3]);
		assert_eq!(ranges.len(), 3);
		assert_eq!(ranges, vec![data1, data2, data3]);

		Ok(())
	}
}
