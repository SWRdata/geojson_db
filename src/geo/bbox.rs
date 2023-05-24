use geojson::Feature;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

// GeoBBox struct representing a geographical bounding box with min and max coordinates
#[derive(Clone, Serialize, Deserialize)]
pub struct GeoBBox {
	x_min: f32, // minimum x-coordinate
	x_max: f32, // maximum x-coordinate
	y_min: f32, // minimum y-coordinate
	y_max: f32, // maximum y-coordinate
}

// Implementation of methods for GeoBBox struct
impl GeoBBox {
	// Create a new GeoBBox given min and max coordinates
	pub fn new(x_min: f32, x_max: f32, y_min: f32, y_max: f32) -> Self {
		GeoBBox {
			x_min,
			x_max,
			y_min,
			y_max,
		}
	}
	// Create an "empty" GeoBBox with extreme values
	pub fn new_empty() -> Self {
		GeoBBox {
			x_min: f32::MAX,
			x_max: f32::MIN,
			y_min: f32::MAX,
			y_max: f32::MIN,
		}
	}
	// Create a GeoBBox from a 1D vector, treating both x and y as same
	fn from_vec(v1: &[f64]) -> Self {
		GeoBBox {
			x_min: v1[0] as f32,
			x_max: v1[0] as f32,
			y_min: v1[1] as f32,
			y_max: v1[1] as f32,
		}
	}
	// Create a GeoBBox from a 2D vector, encapsulating all points in the vector
	fn from_vec2(v2: &[Vec<f64>]) -> Self {
		let mut bbox = GeoBBox::new_empty();
		v2.iter().for_each(|v1| bbox.include_point(v1[0] as f32, v1[1] as f32));
		bbox
	}
	// Create a GeoBBox from a 3D vector, encapsulating all points in the vector
	fn from_vec3(v3: &[Vec<Vec<f64>>]) -> Self {
		let mut bbox = GeoBBox::new_empty();
		v3.iter().for_each(|v2| bbox.include_bbox(&GeoBBox::from_vec2(v2)));
		bbox
	}
	// Create a GeoBBox from a 4D vector, encapsulating all points in the vector
	fn from_vec4(v4: &[Vec<Vec<Vec<f64>>>]) -> Self {
		let mut bbox = GeoBBox::new_empty();
		v4.iter().for_each(|v3| bbox.include_bbox(&GeoBBox::from_vec3(v3)));
		bbox
	}
	// Create a GeoBBox from a geojson String
	pub fn from_geojson(line: &str) -> Self {
		let feature = Feature::from_str(line).unwrap();
		GeoBBox::from_geometry(&feature.geometry.unwrap())
	}
	// Create a GeoBBox from a geojson::Geometry value
	pub fn from_geometry(geometry: &geojson::Geometry) -> Self {
		match &geometry.value {
			geojson::Value::Point(c) => Self::from_vec(c),
			geojson::Value::MultiPoint(c) => Self::from_vec2(c),
			geojson::Value::LineString(c) => Self::from_vec2(c),
			geojson::Value::MultiLineString(c) => Self::from_vec3(c),
			geojson::Value::Polygon(c) => Self::from_vec3(c),
			geojson::Value::MultiPolygon(c) => Self::from_vec4(c),
			geojson::Value::GeometryCollection(c) => {
				let mut bbox = GeoBBox::new_empty();
				c.iter()
					.for_each(|geometry| bbox.include_bbox(&GeoBBox::from_geometry(geometry)));
				bbox
			}
		}
	}
	// Include a point into the GeoBBox, potentially extending it
	fn include_point(&mut self, x: f32, y: f32) {
		if self.x_min > x {
			self.x_min = x
		}
		if self.x_max < x {
			self.x_max = x
		}
		if self.y_min > y {
			self.y_min = y
		}
		if self.y_max < y {
			self.y_max = y
		}
	}
	// Expand current GeoBBox to include another GeoBBox
	pub fn include_bbox(&mut self, bbox: &GeoBBox) {
		if self.x_min > bbox.x_min {
			self.x_min = bbox.x_min
		}
		if self.x_max < bbox.x_max {
			self.x_max = bbox.x_max
		}
		if self.y_min > bbox.y_min {
			self.y_min = bbox.y_min
		}
		if self.y_max < bbox.y_max {
			self.y_max = bbox.y_max
		}
	}
	// Check if GeoBBox is wider than high
	pub fn is_horizontal(&self) -> bool {
		(self.x_max - self.x_min) > (self.y_max - self.y_min)
	}
	// Check if current GeoBBox overlaps with another GeoBBox
	pub fn overlap_bbox(&self, bbox: &GeoBBox) -> bool {
		if self.x_min > bbox.x_max {
			return false;
		}
		if self.x_max < bbox.x_min {
			return false;
		}
		if self.y_min > bbox.y_max {
			return false;
		}
		if self.y_max < bbox.y_min {
			return false;
		}
		true
	}
	// Compute the sum of x coordinates
	pub fn sum_x(&self) -> f32 {
		self.x_min + self.x_max
	}
	// Compute the sum of y coordinates
	pub fn sum_y(&self) -> f32 {
		self.y_min + self.y_max
	}
}
