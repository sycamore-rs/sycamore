//! Easing functions.

use std::f32::consts::PI;

const EXP_BASE: f32 = 2.0;
const BOUNCE_GRAVITY: f32 = 2.75;
const BOUNCE_AMPLITUDE: f32 = 7.5625;

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
    1.0 - f32::sqrt(1.0 - f32::powi(t, 2))
}

pub fn circ_out(t: f32) -> f32 {
    f32::sqrt(1.0 - f32::powi(t - 1.0, 2).powi(2))
}

pub fn circ_inout(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - f32::sqrt(1.0 - f32::powi(2.0 * t, 2))) / 2.0
    } else {
        (f32::sqrt(1.0 - f32::powi(-2.0 * t + 2.0, 2)) + 1.0) / 2.0
    }
}

// Exponential

pub fn expo_in(t: f32) -> f32 {
    if t.abs() <= f32::EPSILON {
        0.0
    } else {
        EXP_BASE.powf(10.0 * t - 10.0)
    }
}

pub fn expo_out(t: f32) -> f32 {
    if (t - 1.0).abs() <= f32::EPSILON {
        1.0
    } else {
        1.0 - EXP_BASE.powf(-10.0 * t)
    }
}

pub fn expo_inout(t: f32) -> f32 {
    if t.abs() <= f32::EPSILON {
        0.0
    } else if (t - 1.0) <= f32::EPSILON {
        1.0
    } else if t <= 0.5 {
        f32::powf(EXP_BASE, 20.0 * t - 10.0) / 2.0
    } else {
        1.0 + f32::powf(EXP_BASE, -20.0 * t + 10.0) / -2.0
    }
}

// Sine

pub fn sine_in(t: f32) -> f32 {
    1.0 - f32::cos(t * PI / 2.0)
}

pub fn sine_out(t: f32) -> f32 {
    f32::sin(t * PI / 2.0)
}

pub fn sine_inout(t: f32) -> f32 {
    -(f32::cos(PI * t) - 1.0) / 2.0
}

// Bounce

pub fn bounce_in(t: f32) -> f32 {
    1.0 - bounce_out(1.0 - t)
}

pub fn bounce_out(t: f32) -> f32 {
    if t < 1.0 / BOUNCE_GRAVITY {
        BOUNCE_AMPLITUDE * t * t
    } else if t < 2.0 / BOUNCE_GRAVITY {
        let t = t - 1.5 / BOUNCE_GRAVITY;
        BOUNCE_AMPLITUDE * t * t + 0.75
    } else if t < 2.5 / BOUNCE_GRAVITY {
        let t = t - 2.25 / BOUNCE_GRAVITY;
        BOUNCE_AMPLITUDE * t * t + 0.9375
    } else {
        let t = t - 2.625 / BOUNCE_GRAVITY;
        BOUNCE_AMPLITUDE * t * t + 0.984375
    }
}

pub fn bounce_inout(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - bounce_out(1.0 - 2.0 * t)) / 2.0
    } else {
        (1.0 + bounce_out(-1.0 + 2.0 * t)) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_start_at_0 {
        ($($ease_fn:ident),*) => {
            paste::paste! {
                $(
                    #[test]
                    fn [<test_ease_ $ease_fn _starts_at_0>]() {
                        assert!(f32::abs($ease_fn(0.0) - 0.0) < f32::EPSILON);
                    }
                )*
            }
        }
    }

    macro_rules! test_end_at_1 {
        ($($ease_fn:ident),*) => {
            paste::paste! {
                $(
                    #[test]
                    fn [<test_ease_ $ease_fn _ends_at_1>]() {
                        assert!(f32::abs($ease_fn(1.0) - 1.0) < f32::EPSILON);
                    }
                )*
            }
        }
    }

    test_start_at_0![
        linear,
        quad_in,
        quad_out,
        quad_inout,
        cubic_in,
        cubic_out,
        cubic_inout,
        quart_in,
        quart_out,
        quart_inout,
        quint_in,
        quint_out,
        quint_inout,
        circ_in,
        circ_out,
        circ_inout,
        expo_in,
        expo_out,
        expo_inout,
        sine_in,
        sine_out,
        sine_inout,
        bounce_in,
        bounce_out,
        bounce_inout
    ];

    test_end_at_1![
        linear,
        quad_in,
        quad_out,
        quad_inout,
        cubic_in,
        cubic_out,
        cubic_inout,
        quart_in,
        quart_out,
        quart_inout,
        quint_in,
        quint_out,
        quint_inout,
        circ_in,
        circ_out,
        circ_inout,
        expo_in,
        expo_out,
        expo_inout,
        sine_in,
        sine_out,
        sine_inout,
        bounce_in,
        bounce_out,
        bounce_inout
    ];
}
