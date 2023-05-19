"use strict";

const Geofile = require('.');
console.log(Geofile);

let file = new Geofile('./data/points.geojsonl')
console.log(file);

let bbox = [13.324571, 52.513766, 13.345771, 52.524473];
console.log(file.find(bbox));
