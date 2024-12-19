use models::{Card, gen_card};
use gfx::{Commands};
use platform_types::{command, sprite, unscaled, Button, Input, Speaker, SFX};
use xs::{Xs, Seed};

#[derive(Clone, Default)]
pub struct Splat {
    pub kind: Card,
    pub x: unscaled::X,
    pub y: unscaled::Y,
}

#[derive(Clone, Default)]
pub struct State {
    pub rng: Xs,
    pub splats: Vec<Splat>,
}

impl State {
    pub fn new(seed: Seed) -> State {
        let rng = xs::from_seed(seed);

        let mut output = State {
            rng,
            .. <_>::default()
        };

        output.add_splat();

        output
    }

    pub fn add_splat(&mut self) {
        let rng = &mut self.rng;

        let kind: Card = gen_card(rng);
        let x = unscaled::X(xs::range(rng, 0..command::WIDTH as u32) as command::Inner);
        let y = unscaled::Y(xs::range(rng, 0..command::HEIGHT as u32) as command::Inner);

        self.splats.push(Splat {
            kind,
            x,
            y,
        });
    }
}

fn update(state: &mut State, input: Input, speaker: &mut Speaker) {
    if input.gamepad != <_>::default() {
        state.add_splat();
        speaker.request_sfx(SFX::CardPlace);
    }
}

#[inline]
fn render(commands: &mut Commands, state: &State) {
    for &Splat { kind, x, y } in &state.splats {
        commands.draw_card(kind, x, y);

        commands.sspr(
            sprite::XY {
                x: sprite::X(0),
                y: sprite::Y(64),
            },
            command::Rect::from_unscaled(unscaled::Rect {
                x: x.saturating_sub(unscaled::W(16)),
                y: y.saturating_sub(unscaled::H(16)),
                w: unscaled::W(16),
                h: unscaled::H(16),
            })
        );
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