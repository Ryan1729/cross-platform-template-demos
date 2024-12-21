use gfx::{Commands};
use platform_types::{colours, command, unscaled, Input, Speaker};
use xs::{Xs, Seed};
use std::f32::consts::TAU;
use std::sync::LazyLock;

const PARTICLE_COUNT: usize = 1 << 12;

fn heart_tau(t: f32) -> (f32, f32) {
    // Based on this parametric equation
    // x = ((sin(t)^3) / 2) + 0.5
    // y = ((13cos(t) - 5cos(2t) - 2cos(3t) - cos(4t)) / 32) + 0.5
    let sin_t = f32::sin(t);
    let x = ((sin_t * sin_t * sin_t) / 2.) + 0.5;
    // TODO? Bake the divide into the coeffiecents, to avoid computing the divide
    let y = ((13. * f32::cos(t) - 5. * f32::cos(2. * t) - 2. * f32::cos(3. * t) - f32::cos(4. * t)) / 32.0) + 0.5;

    (x, 1. - (y + 0.0625))
}

fn heart_01(t: f32) -> (f32, f32) {
    heart_tau(t * TAU)
}

static TARGETS: LazyLock<[unscaled::XY; PARTICLE_COUNT]> = LazyLock::new(|| {
    let mut output: [unscaled::XY; PARTICLE_COUNT] = [<_>::default(); PARTICLE_COUNT];

    for i in 0..PARTICLE_COUNT {
        let t = i as f32 / PARTICLE_COUNT as f32;

        let (heart_x, heart_y) = heart_01(t);

        let Ok(w) = unscaled::const_try_w_from_f32(command::WIDTH as f32 * heart_x) else {
            break
        };
        output[i].x += w;

        let Ok(h) = unscaled::const_try_h_from_f32(command::HEIGHT as f32 * heart_y) else {
            break
        };
        output[i].y += h;
    }

    output
});

fn xs_xy(rng: &mut Xs) -> unscaled::XY {
    unscaled::XY {
        x: unscaled::X(xs::range(rng, 0..command::WIDTH as u32) as command::Inner),
        y: unscaled::Y(xs::range(rng, 0..command::HEIGHT as u32) as command::Inner),
    }
}

#[derive(Clone, Default)]
pub struct Particle {
    pub start: unscaled::XY,
    pub frac: f32,
    pub target: unscaled::XY,
}

#[derive(Clone, Default)]
pub struct State {
    pub rng: Xs,
    pub particles: Vec<Particle>,
}

impl State {
    pub fn new(seed: Seed) -> State {
        let mut rng = xs::from_seed(seed);

        let mut particles = Vec::with_capacity(PARTICLE_COUNT);

        for i in 0..PARTICLE_COUNT {
            let start = xs_xy(&mut rng);

            particles.push(Particle {
                start,
                target: *&TARGETS[i],
                .. <_>::default()
            })
        }

        State {
            rng,
            particles,
            .. <_>::default()
        }
    }
}

fn update(state: &mut State, input: Input, speaker: &mut Speaker) {
    if input.gamepad != <_>::default() {
        *state = State::new(xs::new_seed(&mut state.rng));
        return
    }

    for &mut Particle { ref mut frac, .. } in &mut state.particles {
        if *frac >= 0.0 && *frac < 1.0 {
            *frac += 1./256.;
        } else {
            *frac = 1.;
        }
    }
}

fn lerp(a: f32, t: f32, b: f32) -> f32 {
    a * (1. - t) + b * t
}

#[cfg(test)]
mod lerp_works {
    use super::*;

    #[test]
    fn on_these_examples() {
        // floating point espilon blah blah blah
        // We can just be exact, until that becomes an issue.
        assert_eq!(lerp(4., 1./4., 8.), 5.);
        assert_eq!(lerp(4., -1./4., 8.), 3.);
        assert_eq!(lerp(4., 5./4., 8.), 9.);
    }
}

/// From https://easings.net/#easeInOutBack
fn ease_in_out_back(frac: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C2: f32 = C1 * 1.525;

    if frac < 0.5 {
        ((2. * frac).powf(2.) * ((C2 + 1.) * 2. * frac - C2)) / 2.
    } else {
        ((2. * frac - 2.).powf(2.) * ((C2 + 1.) * (frac * 2. - 2.) + C2) + 2.) / 2.
    }
}

#[inline]
fn render(commands: &mut Commands, state: &State) {
    for &Particle { start, frac, target } in &state.particles {
        let t = ease_in_out_back(frac);

        let at = unscaled::XY {
            x: <_>::try_from(
                lerp(start.x.into(), t, target.x.into())
            ).unwrap_or(target.x),
            y: <_>::try_from(
                lerp(start.y.into(), t, target.y.into())
            ).unwrap_or(target.y),
        };

        //commands.draw_point(target, colours::BLUE);
        commands.draw_point(at, colours::RED);
    }
}

#[inline]
pub fn update_and_render(
    commands: &mut Commands,
    state: &mut State,
    input: Input,
    speaker: &mut Speaker,
) {
    update(state, input, speaker);
    render(commands, state);
}