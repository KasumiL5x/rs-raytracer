# RS Ray Tracer

> A ray tracer implemented in Rust based on https://raytracing.github.io/books/RayTracingInOneWeekend.html.

The ray tracer is implemented as a standalone module that can write to a file or copy to an SDL2 texture.
An SDL window handles the display and keystrokes for running the ray tracer and saving the image.

`Escape`: Quit.

`Space`: Run the ray tracer and update the preview with its result.

`S`: Save the current ray tracer buffer to a PPM file. This is local to your terminal CWD or exe if run directly.
