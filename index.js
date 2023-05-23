"use strict";

import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const { geofileOpen, geofileFind } = require('./index.node');

export default class Geofile {
	#me;
	constructor(filename, memory_size = 64 * 1024 * 1024) {
		this.#me = geofileOpen(filename, memory_size);
	}

	* find(bbox) {
		if ((!Array.isArray(bbox)) || (bbox.length !== 4)) throw Error('argument "bbox" must be an Array of 4 numbers');
		let index = 0;
		const maxCount = 1000;

		do {
			let result = geofileFind.call(this.#me, bbox, index, maxCount);
			let entries = JSON.parse(result[0]);
			index = result[1];
			for (let entry of entries) yield entry;
		} while (index > 0);
	}
}
