//! Viewport WGSL, moved verbatim from viewport.rs.
//! Provenance: our existing shaders, from public graphics literature.

pub(super) const SHADER: &str = "
// The verdict / selection palette rides the uniform (M13.4's
// colour-vision remap; Off = the literals these fields replaced).
struct U { mvp: mat4x4<f32>, eye: vec3<f32>, wire: f32,
           clean: vec4<f32>, rest: vec4<f32>, gouge: vec4<f32>,
           unjudged: vec4<f32>, hot: vec4<f32>, warm: vec4<f32>,
           wire_ink: vec4<f32>, vp: vec4<f32> };
@group(0) @binding(0) var<uniform> u: U;

struct VSOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) sel: f32,
    @location(1) world: vec3<f32>,
    @location(2) cls: f32,
};

@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) sel: f32,
           @location(2) cls: f32) -> VSOut {
    var out: VSOut;
    out.clip = u.mvp * vec4<f32>(pos, 1.0);
    out.sel = sel;
    out.world = pos;
    out.cls = cls;
    return out;
}

// Flat shading: STL/OBJ facets carry no smooth normals, so recover the
// face normal from screen-space derivatives of the world position.
// Key + fill give the broad gradient; a view-dependent specular adds the
// CAD-style sheen that makes facet angles readable at a glance.
// `open` scales the light: 1 on the plain face pass, the baked occlusion
// value of the vertex on the occluded face pass (#2834, `shade_pass`).
fn lit(world: vec3<f32>, sel: f32, cls: f32, open: f32) -> vec4<f32> {
    let cr = cross(dpdx(world), dpdy(world));
    let len = length(cr);
    let n = select(vec3<f32>(0.0, 0.0, 1.0), cr / len, len > 1e-9);
    let key_dir = normalize(vec3<f32>(0.45, 0.60, 0.66));
    // abs(): two-sided, so flipped windings never render unlit-black.
    let key = abs(dot(n, key_dir));
    let fill = abs(dot(n, normalize(vec3<f32>(-0.55, 0.25, 0.40))));
    let light = (0.34 + 0.52 * key + 0.18 * fill) * open;
    let to_eye = normalize(u.eye - world);
    let halfv = normalize(key_dir + to_eye);
    let spec = pow(max(abs(dot(n, halfv)), 0.0), 24.0) * 0.35 * key * open;
    let base = vec3<f32>(0.66, 0.70, 0.76) * light + vec3<f32>(spec);
    let hot = vec3<f32>(0.98, 0.72, 0.20) * light + vec3<f32>(spec);
    // #339 slice 4: a fixture the selected op strikes wears the gouge
    // colour (class STRUCK_CLASS; plain models carry class 0).
    let struck = step(3.5, cls);
    let hit = u.gouge.xyz * light + vec3<f32>(spec);
    return vec4<f32>(mix(mix(base, hot, sel), hit, struck * 0.85), 1.0);
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    return lit(in.world, in.sel, in.cls, 1.0);
}

// Facet edges: dark over the shaded pass, warm when the part is selected.
// In wireframe mode the same wire runs over the ground instead of over
// lit metal, so it brightens to a cool gray on the dark ground and darkens
// to a slate on the light one (#698, `u.wire_ink`; the selection warm
// lifts to the same hot amber the fills use) - `u.wire` is the mode flag.
@fragment
fn fs_edge(in: VSOut) -> @location(0) vec4<f32> {
    let dark = vec3<f32>(0.06, 0.07, 0.09);
    let gray = u.wire_ink.xyz;
    let unlit = mix(dark, gray, u.wire);
    let sel = mix(u.warm.xyz, u.hot.xyz, u.wire);
    return vec4<f32>(mix(unlit, sel, in.sel), 1.0);
}

// Hidden edges (#2818): the same wire where it lies behind a face, blended
// thin over the lit metal so that it reads lighter than the front wire.
@fragment
fn fs_edge_hidden(in: VSOut) -> @location(0) vec4<f32> {
    let dark = vec3<f32>(0.06, 0.07, 0.09);
    return vec4<f32>(mix(dark, u.warm.xyz, in.sel), 0.5);
}

// Stock: same flat shading, cool steel tint, translucent - the model must
// read through the raw material. Selected stock is warmer and denser.
// The `cls` channel carries the deviation verdict (stock_sim's law: 0
// clean, 1 rest, 2 gouge, 3 unjudged) and paints it (our own display
// colors, the same reasoning as REGION_RGB's note): amber material left
// behind, loud red material cut too deep, flat gray where the kernel
// rendered no verdict - NaN is never green, never a clean bill. Plain
// stock parts upload cls = 0 everywhere and look exactly as before.
// Verdict columns keep their color even when selected (the amber lift is
// only for steel), and judged columns sit denser so the color reads.
// Class 4 is no verdict but the incoming stock a carried setup starts
// from (#406): a fixed teal, denser than steel, readable on the light and
// the dark viewport alike (our own display choice).
@fragment
fn fs_stock(in: VSOut) -> @location(0) vec4<f32> {
    let cr = cross(dpdx(in.world), dpdy(in.world));
    let len = length(cr);
    let n = select(vec3<f32>(0.0, 0.0, 1.0), cr / len, len > 1e-9);
    let key_dir = normalize(vec3<f32>(0.45, 0.60, 0.66));
    let key = abs(dot(n, key_dir));
    let light = 0.55 + 0.45 * key;
    var tint = u.clean.xyz;
    var alpha = mix(0.30, 0.55, in.sel);
    switch (i32(in.cls + 0.5)) {
        case 1: { tint = u.rest.xyz; alpha = 0.75; }  // rest
        case 2: { tint = u.gouge.xyz; alpha = 0.85; }  // gouge
        case 3: { tint = u.unjudged.xyz; alpha = 0.25; }  // no verdict
        case 4: { tint = vec3<f32>(0.12, 0.66, 0.74); alpha = 0.45; }  // incoming stock (#406)
        default: {}
    }
    let judged = step(0.5, in.cls);
    let base = mix(u.clean.xyz, u.hot.xyz, in.sel * (1.0 - judged));
    return vec4<f32>(mix(base, tint, judged) * light, alpha);
}

// Cut-region fill: the selected operation's material area painted, not just
// outlined. A constant translucent warning orange (REGION_RGB in the app
// layer, alpha 0.22 - our own display choice) with no lighting: the fill
// marks a level, it is not a surface.
@fragment
fn fs_region(in: VSOut) -> @location(0) vec4<f32> {
    return vec4<f32>(0.95, 0.58, 0.15, 0.22);
}

";
