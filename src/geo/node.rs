use super::GeoBBox;
use serde::{Deserialize, Serialize};

/// The GeoNode struct represents a node in a tree structure that is used for spatial indexing.
#[derive(Serialize, Deserialize, Clone)]
pub struct GeoNode {
	/// The bounding box of the geographic area covered by this node.
	pub bbox: GeoBBox,
	/// Indicates whether this node is a leaf node (true) or not (false).
	pub is_leaf: bool,
	/// For nodes, this represents the index to the left child. For leaves, it's the offset in the file.
	pub value1: usize,
	/// For nodes, this represents the index to the right child. For leaves, it's the length in the file.
	pub value2: usize,
	/// Index to the next sibling node. This refers to the next node in the traversal order that is not part of the current branch.
	///
	/// The following rules are used to determine the next sibling:
	/// 1. Check if the current node is a left or right child of its parent.
	///    - If the current node is a left child, then the "next sibling" is the right child of the parent. In this case, we're done.
	///    - If the current node is a right child, move upwards by treating the parent of the current node as the new current node, and then repeat the process.
	/// 2. If no "next sibling" can be found using the steps above, set 'next' to 0. This value signifies the end of the traversal.
	///
	/// This field aids in the efficient traversal of the tree structure.
	pub next: usize,
}

impl GeoNode {
	/// Creates a new leaf node.
	///
	/// # Arguments
	///
	/// * `bbox` - A GeoBBox instance representing the bounding box of the geographic area.
	/// * `start` - The offset in the file for this leaf node.
	/// * `length` - The length in the file for this leaf node.
	///
	/// # Returns
	///
	/// A new GeoNode instance.
	pub fn new_leaf(bbox: GeoBBox, start: usize, length: usize) -> Self {
		Self {
			bbox,
			is_leaf: true,
			value1: start,
			value2: length,
			next: 0,
		}
	}

	/// Creates a new empty node.
	///
	/// # Arguments
	///
	/// * `bbox` - A GeoBBox instance representing the bounding box of the geographic area.
	///
	/// # Returns
	///
	/// A new GeoNode instance.
	pub fn new_node(bbox: GeoBBox) -> Self {
		Self {
			bbox,
			is_leaf: false,
			value1: 0,
			value2: 0,
			next: 0,
		}
	}
}
