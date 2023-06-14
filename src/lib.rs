mod geo;

use geo::{GeoBBox, GeoDB, GeoFileOptions};
use neon::{
	context::Context,
	prelude::{FunctionContext, ModuleContext, Object},
	result::{JsResult, NeonResult},
	types::{JsArray, JsBox, JsNumber, JsObject, JsString},
};
use std::{cell::RefCell, path::PathBuf, str::from_utf8};

type BoxedGeoDB = JsBox<RefCell<GeoDB>>;

impl GeoDB {
	pub fn js_open(mut cx: FunctionContext) -> JsResult<BoxedGeoDB> {
		let filename = PathBuf::from(cx.argument::<JsString>(0)?.value(&mut cx));
		let options = cx.argument::<JsObject>(1).unwrap_or(cx.empty_object());

		let get_usize = |name: &str, cx: &mut FunctionContext| -> Option<usize> {
			options
				.get_opt::<JsNumber, _, _>(cx, name)
				.unwrap()
				.map(|v| v.value(cx) as usize)
		};

		let opt = GeoFileOptions {
			separator: options
				.get_opt::<JsString, _, _>(&mut cx, "separator")?
				.map(|v| v.value(&mut cx)),
			col_x: get_usize("colX", &mut cx),
			col_y: get_usize("colY", &mut cx),
			skip_lines: get_usize("skipLines", &mut cx),
		};

		match GeoDB::open(&filename, opt) {
			Ok(geo_file) => Ok(cx.boxed(RefCell::new(geo_file))),
			Err(err) => cx.throw_error(err.to_string()),
		}
	}
	pub fn js_find(mut cx: FunctionContext) -> JsResult<JsArray> {
		let geo_db_js = cx.this().downcast_or_throw::<BoxedGeoDB, _>(&mut cx)?;
		let geo_db = geo_db_js.borrow();

		let bbox = cx.argument::<JsArray>(0)?.to_vec(&mut cx)?;
		let bbox: Vec<f32> = bbox
			.iter()
			.map(|v| v.downcast_or_throw::<JsNumber, _>(&mut cx).unwrap().value(&mut cx) as f32)
			.collect();
		let bbox = GeoBBox::new(bbox[0], bbox[2], bbox[1], bbox[3]);

		let start_index = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
		let max_count = cx.argument::<JsNumber>(2)?.value(&mut cx) as usize;

		let (entries, next_index) = geo_db.query_bbox(&bbox, start_index, max_count).unwrap();
		let array = cx.empty_array();

		for (i, entry) in entries.iter().enumerate() {
			let line = cx.string(from_utf8(entry).unwrap());
			array.set(&mut cx, i as u32, line)?;
		}

		let next_index = cx.number(next_index as u32);
		let n = array.len(&mut cx);
		array.set(&mut cx, n, next_index)?;

		Ok(array)
	}
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
	cx.export_function("geofileOpen", GeoDB::js_open)?;
	cx.export_function("geofileFind", GeoDB::js_find)?;
	Ok(())
}
