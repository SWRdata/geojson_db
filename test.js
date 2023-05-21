"use strict";

import Geofile from './index.js';
console.log(Geofile);

let file = new Geofile('./data/points.geojsonl')
console.log(file);

//let bbox = [13.324571, 52.513766, 13.345771, 52.524473];
let bbox = [13.3, 52.5, 13.4, 52.6];
console.log(file.find(bbox));
