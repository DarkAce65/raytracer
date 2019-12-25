# raytracing

A ray tracer written in Rust which implements Blinn-Phong shading and physically-based rendering using a Metallic-Roughness workflow

### Building/Running

*Make sure Rust and `cargo` are installed. Note that the following commands compile the ray tracer in release mode for maximum optimization.*

- For a live visualization of the ray tracer, run `cargo run --release -- scenes/scene.json`
- To output to a file, run `cargo run --release -- -o image.png scenes/scene.json`

*Additional scenes can be found in the [scenes](./scenes) folder*

----

#### raytrace usage

The following options may be passed through `cargo` like so: `cargo run --release -- [FLAGS] [OPTIONS] <scene>`

```
ray tracer

USAGE:
    raytrace [FLAGS] [OPTIONS] <scene>

FLAGS:
    -h, --help           Prints help information
        --no-progress    Hide progress bar
    -V, --version        Prints version information

OPTIONS:
    -o, --output <output>    Output rendered image to file, ray tracer outputs to a window if --output is omitted

ARGS:
    <scene>    Input scene as a json file
```
