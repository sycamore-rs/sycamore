//! Easing functions.

// Linear

pub fn linear(t: f32) -> f32 {
    t
}

// Quadratic

pub fn quad_in(t: f32) -> f32 {
    t * t
}

pub fn quad_out(t: f32) -> f32 {
    -t * (t - 2.0)
}

pub fn quad_inout(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        -2.0 * t * t + 4.0 * t - 1.0
    }
}

// Cubic

pub fn cubic_in(t: f32) -> f32 {
    t * t * t
}

pub fn cubic_out(t: f32) -> f32 {
    let f = t - 1.0;
    f * f * f + 1.0
}

pub fn cubic_inout(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let f = 2.0 * t - 2.0;
        0.5 * f * f * f + 1.0
    }
}

// Quartic

pub fn quart_in(t: f32) -> f32 {
    t * t * t * t
}

pub fn quart_out(t: f32) -> f32 {
    let f = t - 1.0;
    f * f * f * (1.0 - t) + 1.0
}

pub fn quart_inout(t: f32) -> f32 {
    if t < 0.5 {
        8.0 * t * t * t * t
    } else {
        let f = t - 1.0;
        -8.0 * f * f * f * f + 1.0
    }
}

// Quintic

pub fn quint_in(t: f32) -> f32 {
    t * t * t * t * t
}

pub fn quint_out(t: f32) -> f32 {
    let f = t - 1.0;
    f * f * f * f * f + 1.0
}

pub fn quint_inout(t: f32) -> f32 {
    if t < 0.5 {
        16.0 * t * t * t * t * t
    } else {
        let f = (2.0 * t) - 2.0;
        0.5 * f * f * f * f * f + 1.0
    }
}

// TODO: add more easing functions
