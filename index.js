"use strict";

import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const { geofileOpen, geofileFind } = require('./index.node');

export default class Geofile {
	#me;
	constructor(filename, memory_size = 64 * 1024 * 1024) {
		console.log({ filename, memory_size });
		this.#me = geofileOpen(filename, memory_size);
	}

	* find(bbox) {
		if ((!Array.isArray(bbox)) || (bbox.length !== 4)) throw Error('argument "bbox" must be an Array of 4 numbers');
		let index = 0;
		const maxCount = 10000;

		do {
			let result = geofileFind.call(this.#me, bbox, index, maxCount);
			//console.log(result[0]);
			//console.log(result[0].toString());
			let entries = JSON.parse(result[0]);
			for (let entry of entries) yield entry;
			index = result[1];
		} while (index > 0);
	}
}
