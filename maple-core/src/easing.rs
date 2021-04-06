//! Easing functions.

use core::f32;

const EXP_BASE: f32 = 2.;

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

// Circular

pub fn circ_in(t: f32) -> f32 {
    1. - (1. - t.powi(2)).sqrt()
}

pub fn circ_out(t: f32) -> f32 {
    (1. - (t - 1.).powi(2)).sqrt()
}

pub fn circ_inout(t: f32) -> f32 {
    if t < 0.5 {
        circ_in(2.*t) / 2.
    } else {
        (1. - (-2.*t + 2.).powi(2)).sqrt() / 2.
    }
}

// Exponential

pub fn expo_in(t: f32) -> f32 {
    if t.abs() <= 0.0002 {0.}
    else {EXP_BASE.powf(10.*t - 10.)}
}

pub fn expo_out(t: f32) -> f32 {
    if (t - 1.).abs() <= 0.0002 {0.}
    else {1. - EXP_BASE.powf(-10.*t)}
}

pub fn expo_inout(t: f32) -> f32 {
    if t.abs() <= 0.0002 {0.}
    else if (t - 1.).abs() <= 0.0002 {1.}
    else if t <= 0.5 {EXP_BASE.powf(20.*t - 10.) / 2.}
    else {2. - EXP_BASE.powf(-20.*t + 10.) / 2.}
}

// TODO: add more easing functions
