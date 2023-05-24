use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct GeoBBox {
	x_min: f32,
	x_max: f32,
	y_min: f32,
	y_max: f32,
}

impl GeoBBox {
	pub fn new(x_min: f32, x_max: f32, y_min: f32, y_max: f32) -> Self {
		GeoBBox {
			x_min,
			x_max,
			y_min,
			y_max,
		}
	}
	pub fn new_empty() -> Self {
		GeoBBox {
			x_min: f32::MAX,
			x_max: f32::MIN,
			y_min: f32::MAX,
			y_max: f32::MIN,
		}
	}
	fn from_vec(v1: &[f64]) -> Self {
		GeoBBox {
			x_min: v1[0] as f32,
			x_max: v1[0] as f32,
			y_min: v1[1] as f32,
			y_max: v1[1] as f32,
		}
	}
	fn from_vec2(v2: &[Vec<f64>]) -> Self {
		let mut bbox = GeoBBox::new_empty();
		v2.iter().for_each(|v1| bbox.include_point(v1[0] as f32, v1[1] as f32));
		bbox
	}
	fn from_vec3(v3: &[Vec<Vec<f64>>]) -> Self {
		let mut bbox = GeoBBox::new_empty();
		v3.iter().for_each(|v2| bbox.include_bbox(&GeoBBox::from_vec2(v2)));
		bbox
	}
	fn from_vec4(v4: &[Vec<Vec<Vec<f64>>>]) -> Self {
		let mut bbox = GeoBBox::new_empty();
		v4.iter().for_each(|v3| bbox.include_bbox(&GeoBBox::from_vec3(v3)));
		bbox
	}
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
	pub fn is_horizontal(&self) -> bool {
		(self.x_max - self.x_min) > (self.y_max - self.y_min)
	}
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
	pub fn sum_x(&self) -> f32 {
		self.x_min + self.x_max
	}
	pub fn sum_y(&self) -> f32 {
		self.y_min + self.y_max
	}
}
