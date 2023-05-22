mod geo;

use geo::{GeoBBox, GeoDB};
use neon::{
	context::Context,
	prelude::{FunctionContext, ModuleContext, Object},
	result::{JsResult, NeonResult},
	types::{JsArray, JsBox, JsNumber, JsString},
};
use std::{cell::RefCell, path::PathBuf};

type BoxedGeoFile = JsBox<RefCell<GeoDB>>;

impl GeoDB {
	pub fn js_open(mut cx: FunctionContext) -> JsResult<BoxedGeoFile> {
		let filename = PathBuf::from(cx.argument::<JsString>(0)?.value(&mut cx));
		let memory_size = cx
			.argument::<JsNumber>(1)
			.unwrap_or(cx.number(64 * 1204 * 1024))
			.value(&mut cx);

		let geo_file = GeoDB::open(&filename, memory_size as usize).unwrap();
		return Ok(cx.boxed(RefCell::new(geo_file)));
	}
	pub fn js_find(mut cx: FunctionContext) -> JsResult<JsArray> {
		let geo_file = cx.this().downcast_or_throw::<BoxedGeoFile, _>(&mut cx)?;

		let bbox = cx.argument::<JsArray>(0)?.to_vec(&mut cx)?;
		let bbox: Vec<f64> = bbox
			.iter()
			.map(|v| v.downcast_or_throw::<JsNumber, _>(&mut cx).unwrap().value(&mut cx))
			.collect();

		let start_index = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
		let max_count = cx.argument::<JsNumber>(2)?.value(&mut cx) as usize;

		let bbox = GeoBBox::new(bbox[0], bbox[2], bbox[1], bbox[3]);

		let (entries, next_index) = geo_file.borrow_mut().find(&bbox, start_index, max_count).unwrap();
		let n = entries.len() as u32;
		let array = cx.empty_array();
		for (i, entry) in entries.into_iter().enumerate() {
			let line = cx.string(entry);
			array.set(&mut cx, i as u32, line)?;
		}
		let index = cx.number(next_index as u32);
		array.set(&mut cx, n, index)?;
		return Ok(array);
	}
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
	cx.export_function("geofileOpen", GeoDB::js_open)?;
	cx.export_function("geofileFind", GeoDB::js_find)?;
	Ok(())
}
