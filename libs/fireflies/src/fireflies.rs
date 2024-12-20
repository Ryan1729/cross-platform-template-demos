use gfx::{Commands};
use platform_types::{colours, command, unscaled, Input, Speaker};
use xs::{Xs, Seed};

fn xs_xy(rng: &mut Xs) -> unscaled::XY {
    unscaled::XY {
        x: unscaled::X(xs::range(rng, 0..command::WIDTH as u32) as command::Inner),
        y: unscaled::Y(xs::range(rng, 0..command::HEIGHT as u32) as command::Inner),
    }
}

const PARTICLE_COUNT: usize = 1024;

#[derive(Clone, Default)]
pub struct Particle {
    pub at: unscaled::XY,
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
            particles.push(Particle {
                at: xs_xy(&mut rng),
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

    for &mut Particle { ref mut at, ref mut target } in &mut state.particles {
        if at != target {
            // TODO store start and accumulate a 0.0-1.0 value
            // TODO easing function based on accumulated value
            let t_x = 0.5;
            let t_y = 0.5;
            at.x = <_>::try_from(
                lerp(at.x.into(), t_x, target.x.into())
            ).unwrap_or(at.x);
            at.y = <_>::try_from(
                lerp(at.y.into(), t_y, target.y.into())
            ).unwrap_or(at.y);
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

#[inline]
fn render(commands: &mut Commands, state: &State) {
    for &Particle { at, target } in &state.particles {
        commands.draw_point(at, colours::RED);
        commands.draw_point(target, colours::BLUE);
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