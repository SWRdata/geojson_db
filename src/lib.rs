mod geo;

use geo::{GeoBBox, GeoDB};
use neon::{
	context::Context,
	prelude::{FunctionContext, ModuleContext, Object},
	result::{JsResult, NeonResult},
	types::{JsArray, JsBox, JsBuffer, JsNumber, JsString},
};
use std::{
	cell::RefCell,
	io::{BufWriter, Write},
	path::PathBuf,
};

type BoxedGeoDB = JsBox<RefCell<GeoDB>>;

impl GeoDB {
	pub fn js_open(mut cx: FunctionContext) -> JsResult<BoxedGeoDB> {
		let filename = PathBuf::from(cx.argument::<JsString>(0)?.value(&mut cx));

		let _memory_size = cx
			.argument::<JsNumber>(1)
			.unwrap_or(cx.number(64 * 1204 * 1024))
			.value(&mut cx) as usize;

		match GeoDB::open(&filename) {
			Ok(geo_file) => Ok(cx.boxed(RefCell::new(geo_file))),
			Err(err) => cx.throw_error(err.to_string()),
		}
	}
	pub fn js_find(mut cx: FunctionContext) -> JsResult<JsArray> {
		let geo_db_js = cx.this().downcast_or_throw::<BoxedGeoDB, _>(&mut cx)?;
		let geo_db = geo_db_js.borrow();

		let bbox = cx.argument::<JsArray>(0)?.to_vec(&mut cx)?;
		let bbox: Vec<f64> = bbox
			.iter()
			.map(|v| v.downcast_or_throw::<JsNumber, _>(&mut cx).unwrap().value(&mut cx))
			.collect();

		let start_index = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
		let max_count = cx.argument::<JsNumber>(2)?.value(&mut cx) as usize;

		let bbox = GeoBBox::new(bbox[0], bbox[2], bbox[1], bbox[3]);

		let (entries, next_index) = geo_db.query_bbox(&bbox, start_index, max_count).unwrap();
		let array = cx.empty_array();

		let mut buffer = BufWriter::new(vec![]);
		buffer.write_all(b"[").unwrap();
		for (i, entry) in entries.iter().enumerate() {
			if i != 0 {
				buffer.write_all(b",").unwrap();
			}
			buffer.write_all(entry).unwrap();
		}
		buffer.write_all(b"]").unwrap();
		let buffer = JsBuffer::external(&mut cx, buffer.into_inner().unwrap());
		array.set(&mut cx, 0, buffer)?;

		let next_index = cx.number(next_index as u32);
		array.set(&mut cx, 1, next_index)?;

		Ok(array)
	}
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
	cx.export_function("geofileOpen", GeoDB::js_open)?;
	cx.export_function("geofileFind", GeoDB::js_find)?;
	Ok(())
}
