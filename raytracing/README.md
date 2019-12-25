# raytracing

A ray tracer written in Rust which implements Blinn-Phong shading and physically-based rendering using a Metallic-Roughness workflow

### Building/Running

*Make sure Rust and `cargo` are installed.*

- For a live visualization of the ray tracer, run `cargo run -- scenes/scene.json`
- To output to a file, run `cargo run -- -o image.png scenes/scene.json`

*Additional scenes can be found in the [scenes](./scenes) folder*

----

#### raytrace usage

The following options may be passed through `cargo` like so: `cargo run -- [FLAGS] [OPTIONS] <scene>`

```
ray tracer

USAGE:
    raytrace [FLAGS] [OPTIONS] <scene>

FLAGS:
    -h, --help           Prints help information
        --no-progress    Hide progress bar
    -V, --version        Prints version information

OPTIONS:
    -o, --output <output>    Output rendered image to file
                             If omitted, image is rendered to a window

ARGS:
    <scene>    input scene as a json file
```
