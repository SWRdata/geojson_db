use super::GeoBBox;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct GeoNode {
	pub bbox: GeoBBox,
	pub is_leaf: bool,
	// NODE: index to left child
	// LEAF: offset in file
	pub value1: usize,
	// NODE: index to right child
	// LEAF: length in file
	pub value2: usize,
	// index to next sibling
	pub next: usize,
}

impl GeoNode {
	pub fn new_leaf(bbox: GeoBBox, start: usize, length: usize) -> Self {
		Self {
			bbox,
			is_leaf: true,
			value1: start,
			value2: length,
			next: 0,
		}
	}
}
