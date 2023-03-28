# Gaka-rs (画家)

An OpenGL 4.3+ Renderer written in Rust 🦀

![Gaka rendering a Bezier surface in a winit Window](/.readme/bezier_surface.png?raw=true "Gaka rendering a Bezier surface")


## TODO:
- [ ] Bezier curves
    - [x] From a given number of line segments
    - [ ] From a given length of line segment
    - [x] Visualization
- [ ] Bezier surfaces
    - [x] Lambertian reflectance
    - [x] Surface normals visualization
    - [ ] Parameter space
- [ ] Real-time rendering
    - [ ] BRDF Implementation
    - [ ] Texture support
    - [x] Three-point lighting
- [ ] BONUS
    - [ ] Interactive Bezier curve manipulation
    - [ ] Skybox
- [ ] General improvements
    - [ ] Improve GL version handling (currently hard-coded as 4.6)
    - [ ] Add a simple GUI via EGUI or ImGUI
    - [ ] WASM target with WebGL backend !!??