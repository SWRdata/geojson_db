# GeoJSON DB

GeoJSON DB is a high-performance npm package designed to facilitate rapid spatial queries on GeoJSON files. Currently, it supports only `.geojsonl` files (line-delimited GeoJSON Features). Due to its in-memory architecture, ensure you have sufficient memory available to load the file.

This project was initially bootstrapped with [create-neon](https://www.npmjs.com/package/create-neon). 

## Example Usage
```javascript
import Geofile from 'geojson_db';

let file = new Geofile('features.geojsonl');

let bbox = [7, 49, 8, 50];
for (let feature of file.find(bbox)) {
	console.log(feature);
}
```

## Installation 

GeoJSON DB requires a [supported version of Node and Rust](https://github.com/neon-bindings/neon#platform-support).

To install the project using npm, navigate to the project directory, and run:

```sh
$ npm install
```
This command installs the project and its dependencies and also initiates the build.

## Build

To only execute the build process for an already installed project, run:

```sh
$ npm run build
```
This command uses the [cargo-cp-artifact](https://github.com/neon-bindings/cargo-cp-artifact) utility to perform the Rust build and subsequently copies the built library into `./index.node`.

## Available Scripts 

In the project directory, you can run:

* `npm install` - Installs the project, including running `npm run build`.
* `npm build` - Builds the Node addon (`index.node`) from source.

### Cargo build arguments

You can pass additional [`cargo build`](https://doc.rust-lang.org/cargo/commands/cargo-build.html) arguments to `npm build` and `npm build-*` commands. For example, to enable a [cargo feature](https://doc.rust-lang.org/cargo/reference/features.html):

```sh
npm run build -- --feature=beetle
```

* `npm build-debug` - Alias for `npm build`.
* `npm build-release` - Equivalent to `npm build` but builds the module with the [`release`](https://doc.rust-lang.org/cargo/reference/profiles.html#release) profile. Although release builds compile slower, they run faster.
* `npm test` - Executes the unit tests using `cargo test`. To learn more about [adding tests to your Rust code](https://doc.rust-lang.org/book/ch11-01-writing-tests.html), refer to the [Rust book](https://doc.rust-lang.org/book/).

## Project Structure

The project directory is structured as follows:

```
geojson_db/
├── Cargo.toml
├── README.md
├── index.node
├── package.json
├── src/
|   └── lib.rs
└── target/
```

## Documentation and Additional Learning Resources

* To learn more about Neon, visit the [Neon documentation](https://neon-bindings.com).
* To dive deeper into Rust, access the [Rust documentation](https://www.rust-lang.org).
* To explore Node further, see the [Node documentation](https://nodejs.org).