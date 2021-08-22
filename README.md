# raytracer

A ray tracer written in Rust which implements Blinn-Phong shading and physically-based rendering using a Metallic-Roughness workflow.

### References

- [_Physically Based Rendering: From Theory To Implementation_](http://www.pbr-book.org/)
- [Scratchapixel](https://www.scratchapixel.com)
- https://bheisler.github.io/post/writing-raytracer-in-rust-part-1/
- [three.js](https://threejs.org/)

### Building/Running

_Requires that [Rust and `cargo`](https://www.rust-lang.org/learn/get-started) are installed._

- For a live visualization of the ray tracer, run `cargo run -- scenes/scene.json`
- To output to a file, run `cargo run -- -o image.png scenes/scene.json`

Additional sample scene files are in the [`scenes`](./scenes) folder.

### Renders

See the full list of renders [here](./renders/renders.md).

`scenes/scene.json` (800 x 800 pixels, 4 spp, 1,524 primitives, 147,113,202 rays, 49.979s on i7 8650U)

![scene.json](./renders/scene.png)

---

### raytrace usage

The following options may be passed through `cargo` like so: `cargo run -- [FLAGS] [OPTIONS] <scene>`

```
ray tracer
A ray tracer written in Rust

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
