// CRT Screen Post-Processing Shader
// Compatible with Bevy's fullscreen vertex shader pattern.
//
// Effects included:
//   - Barrel/pincushion lens distortion
//   - Electron beam horizontal phosphor blur
//       Each scanline row is blurred along X with a Gaussian whose sigma
//       widens for bright pixels (bright phosphors bloom more).
//       The blur is strictly horizontal, matching the left→right sweep
//       of the electron gun.  A separate, much narrower vertical blur
//       adds the small amount of vertical phosphor spread.
//   - Per-scanline brightness envelope (beam cross-section profile)
//       Inside each row the beam has a smooth bell shape vertically,
//       so the lit band is bright in the centre and fades to black at
//       the top/bottom border — exactly like a real electron beam spot.
//   - Phosphor RGB subpixel mask (aperture grille / shadow mask)
//   - Chromatic aberration
//   - Screen vignette
//   - Temporal noise / static grain
//   - Edge blackout outside the curved screen
//   - Brightness / contrast / saturation

// ---------------------------------------------------------------------------
// Vertex stage  (unchanged from Bevy's fullscreen helper)
// ---------------------------------------------------------------------------

struct FullscreenVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0)       uv:       vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> FullscreenVertexOutput {
    let uv       = vec2<f32>(f32(vertex_index >> 1u), f32(vertex_index & 1u)) * 2.0;
    let position = vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
    return FullscreenVertexOutput(position, uv);
}

// ---------------------------------------------------------------------------
// Bindings
// ---------------------------------------------------------------------------

@group(0) @binding(0) var frame_texture: texture_2d<f32>;
@group(0) @binding(1) var frame_sampler: sampler;

// ---------------------------------------------------------------------------
// Tuning constants
// ---------------------------------------------------------------------------

// Lens distortion: 0 = flat, 0.10 = subtle CRT curve
const CURVATURE: f32             = 0.10;

// Number of CRT scanlines (virtual rows).
// Keep this <= half your vertical resolution so each row spans >=2 pixels
// and the dark gap never collapses to a single aliased black line.
// At 1080p output: 240 gives ~4.5px/row (safe), 320 gives ~3.4px (fine).
const SCANLINE_COUNT: f32        = 240.0;

// Vertical beam profile sharpness.
// 0.5 -> very soft;  2.0 -> visible rows with no harsh black gap.
const BEAM_SHARPNESS: f32        = 1.8;

// The envelope never drops below this value (0 = full black gap, 0.3 = soft dip).
// Raising this removes the trippy black lines while keeping the scanline feel.
const BEAM_FLOOR: f32            = 0.25;

// ---- Horizontal phosphor sweep blur ------------------------------------
// Base Gaussian half-width in UV units (always applied).
// This is the minimum horizontal softness of every pixel.
const BEAM_BLUR_BASE: f32        = 0.0035;

// Extra blur added for bright pixels (phosphor saturation / overload).
// bright_extra = BEAM_BLUR_BRIGHT * luminance(pixel)
// So dark areas stay crisp; bright whites bloom wide horizontally.
const BEAM_BLUR_BRIGHT: f32      = 0.002;

// Number of horizontal taps.  Must be ODD.  11 -> good quality.
const BLUR_TAPS: i32             = 11;

// ---- Vertical phosphor spread (much smaller than horizontal) -----------
const VERT_BLUR: f32             = 0.0008;

// Shadow-mask / phosphor-dot intensity: 0 = off, 0.3 = visible RGB stripes
const MASK_STRENGTH: f32         = 0.22;

// Chromatic aberration (UV units)
const CHROMA_OFFSET: f32         = 0.0018;

// Vignette strength
const VIGNETTE_STRENGTH: f32     = 0.45;

// Static noise
const NOISE_STRENGTH: f32        = 0.030;

// Replace with a per-frame uniform to animate noise / rolling effects
const TIME_SEED: f32             = 1.0;

// Colour grading
const BRIGHTNESS: f32            = 2.5;
const CONTRAST: f32              = 1.08;
const SATURATION: f32            = 1.15;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn hash(p: vec2<f32>) -> f32 {
    var q = vec2<f32>(dot(p, vec2<f32>(127.1, 311.7)),
                      dot(p, vec2<f32>(269.5, 183.3)));
    return fract(sin(dot(q, vec2<f32>(1.0, 1.0))) * 43758.5453123);
}

fn barrel_distort(uv_c: vec2<f32>, k: f32) -> vec2<f32> {
    let r2 = dot(uv_c, uv_c);
    return uv_c * (1.0 + k * r2);
}

fn luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn grade(col: vec3<f32>) -> vec3<f32> {
    var c = col * BRIGHTNESS;
    c = (c - 0.5) * CONTRAST + 0.5;
    let lum = luminance(c);
    c = mix(vec3<f32>(lum), c, SATURATION);
    return clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
}

// Gaussian weight for a given distance and sigma
fn gauss(d: f32, sigma: f32) -> f32 {
    return exp(-0.5 * (d / sigma) * (d / sigma));
}

// ---------------------------------------------------------------------------
// Horizontal phosphor sweep blur
//
// Models the electron beam sweeping left to right across one scanline.
// The phosphor at each point is excited by the beam's Gaussian cross-section.
// Bright pixels excite more phosphor -> wider horizontal glow.
//
// We first take a cheap centre-tap to estimate local brightness, then widen
// sigma before accumulating the weighted horizontal taps.
// ---------------------------------------------------------------------------
fn beam_blur_h(uv: vec2<f32>) -> vec3<f32> {
    // Estimate brightness from the raw centre pixel
    let centre = textureSample(frame_texture, frame_sampler, uv).rgb;
    let lum    = luminance(centre);

    // Sigma is wider for brighter pixels
    let sigma      = BEAM_BLUR_BASE + BEAM_BLUR_BRIGHT * lum;
    let half_taps  = BLUR_TAPS / 2;

    var acc        = vec3<f32>(0.0);
    var weight_sum = 0.0;

    for (var i: i32 = -half_taps; i <= half_taps; i++) {
        // Space taps evenly across ~3 sigma so the kernel covers the glow
        let tap_uv_offset = f32(i) * (sigma / f32(half_taps));
        let w             = gauss(tap_uv_offset, sigma);
        let sample_uv     = uv + vec2<f32>(tap_uv_offset, 0.0);
        acc        += textureSample(frame_texture, frame_sampler, sample_uv).rgb * w;
        weight_sum += w;
    }

    return acc / weight_sum;
}

// Tiny vertical blur — phosphor dots have a small vertical spread too
fn beam_blur_v(uv: vec2<f32>, sigma: f32) -> vec3<f32> {
    var acc        = vec3<f32>(0.0);
    var weight_sum = 0.0;
    for (var i: i32 = -2; i <= 2; i++) {
        let off = f32(i) * sigma;
        let w   = gauss(off, sigma);
        acc        += textureSample(frame_texture, frame_sampler, uv + vec2<f32>(0.0, off)).rgb * w;
        weight_sum += w;
    }
    return acc / weight_sum;
}

// ---------------------------------------------------------------------------
// Fragment shader
// ---------------------------------------------------------------------------

@fragment
fn fs_main(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {

    // --- 1. Lens distortion -------------------------------------------
    let uv_c      = in.uv - 0.5;
    let uv_bent_c = barrel_distort(uv_c, CURVATURE);
    let uv_bent   = uv_bent_c + 0.5;

    let outside   = any(uv_bent < vec2<f32>(0.0)) || any(uv_bent > vec2<f32>(1.0));

    // --- 2. Chromatic aberration  ------------------------------------
    // Offset R and B radially away from centre
    let ca_dir = normalize(uv_bent_c + vec2<f32>(0.0001));
    let uv_r   = uv_bent - ca_dir * CHROMA_OFFSET;
    let uv_b   = uv_bent + ca_dir * CHROMA_OFFSET;

    // --- 3. Horizontal phosphor sweep blur  --------------------------
    // Run the H-blur independently per CA-shifted UV so fringing and
    // horizontal bloom interact correctly.
    let blur_r = beam_blur_h(uv_r).r;
    let blur_g = beam_blur_h(uv_bent).g;
    let blur_b = beam_blur_h(uv_b).b;

    // Small vertical spread blended in
    let blur_v = beam_blur_v(uv_bent, VERT_BLUR);

    var col = vec3<f32>(blur_r, blur_g, blur_b);
    col = mix(col, blur_v, 0.25);

    // --- 4. Beam vertical profile (scanline brightness envelope) -----
    //
    // Within each scanline row the electron beam has a Gaussian
    // cross-section vertically.  We map position within the row to [0,1]
    // where 0 = beam centre (full brightness) and 1 = row edge (black).
    //
    //  scanline_phase in [0,1):  0 = row centre, 0.5 = midpoint to next
    let scanline_phase = fract(uv_bent.y * SCANLINE_COUNT);
    // Distance from nearest row centre in [0, 0.5], normalised to [0, 1]
    let norm_dist      = abs(scanline_phase - 0.5) * 2.0;
    // Bell curve: BEAM_FLOOR lifts the minimum so the gap never goes fully black,
    // eliminating harsh aliased lines at high output resolutions.
    let beam_envelope  = BEAM_FLOOR + (1.0 - BEAM_FLOOR) * pow(1.0 - norm_dist, BEAM_SHARPNESS);
    col                = col * beam_envelope;

    // --- 5. Shadow mask (RGB aperture grille) ------------------------
    let pixel_x  = in.position.x;
    let stripe   = u32(pixel_x) % 3u;
    var mask_col = vec3<f32>(1.0);
    if stripe == 0u {
        mask_col = vec3<f32>(1.0, 1.0 - MASK_STRENGTH, 1.0 - MASK_STRENGTH);
    } else if stripe == 1u {
        mask_col = vec3<f32>(1.0 - MASK_STRENGTH, 1.0, 1.0 - MASK_STRENGTH);
    } else {
        mask_col = vec3<f32>(1.0 - MASK_STRENGTH, 1.0 - MASK_STRENGTH, 1.0);
    }
    col = col * mask_col;

    // --- 6. Vignette -------------------------------------------------
    let vign_r2 = dot(uv_bent_c * 2.0, uv_bent_c * 2.0);
    col = col * (1.0 - VIGNETTE_STRENGTH * smoothstep(0.3, 1.4, vign_r2));

    // --- 7. Noise / grain --------------------------------------------
    let grain = hash((in.uv + vec2<f32>(TIME_SEED * 0.1)) * vec2<f32>(1920.0, 1080.0)) * 2.0 - 1.0;
    col = col + grain * NOISE_STRENGTH;

    // --- 8. Colour grading -------------------------------------------
    col = grade(col);

    // --- 9. Outside the screen -> black ------------------------------
    if outside {
        col = vec3<f32>(0.0);
    }

    return vec4<f32>(col, 1.0);
}
