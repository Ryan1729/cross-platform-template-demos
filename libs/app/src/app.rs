use gfx::{Commands};
use platform_types::{command, sprite, unscaled, Button, Input, Speaker, SFX};
pub use platform_types::StateParams;

enum DemoState {
    Fireflies(fireflies::State),
    Splat(splat::State),
}

pub struct State {
    pub demo_state: DemoState,
    pub commands: Commands,
    pub input: Input,
    pub speaker: Speaker,
}

impl State {
    pub fn new((seed, logger, error_logger): StateParams) -> Self {
        unsafe {
            features::GLOBAL_LOGGER = logger;
            features::GLOBAL_ERROR_LOGGER = error_logger;
        }

        // We always want to log the seed, if there is a logger available, so use the function,
        // not the macro.
        features::log(&format!("{:?}", seed));

        let mut demo_state = DemoState::Fireflies(fireflies::State::new(seed));

        Self {
            demo_state,
            commands: Commands::default(),
            input: Input::default(),
            speaker: Speaker::default(),
        }
    }
}

impl platform_types::State for State {
    fn frame(&mut self) -> (&[platform_types::Command], &[SFX]) {
        self.commands.clear();
        self.speaker.clear();
        update_and_render(
            &mut self.commands,
            &mut self.demo_state,
            self.input,
            &mut self.speaker,
        );

        self.input.previous_gamepad = self.input.gamepad;

        (self.commands.slice(), self.speaker.slice())
    }

    fn press(&mut self, button: Button) {
        if self.input.previous_gamepad.contains(button) {
            //This is meant to pass along the key repeat, if any.
            //Not sure if rewriting history is the best way to do this.
            self.input.previous_gamepad.remove(button);
        }

        self.input.gamepad.insert(button);
    }

    fn release(&mut self, button: Button) {
        self.input.gamepad.remove(button);
    }
}

#[inline]
fn update_and_render(
    commands: &mut Commands,
    state: &mut DemoState,
    input: Input,
    speaker: &mut Speaker,
) {
    use DemoState::*;

    if input.pressed_this_frame(Button::SELECT) {
        match state {
            Fireflies(s) => {
                *state = Splat(splat::State::new(xs::new_seed(&mut s.rng)));
            }
            Splat(s) => {
                *state = Fireflies(fireflies::State::new(xs::new_seed(&mut s.rng)));
            }
        }
    }

    match state {
        Fireflies(s) => {
            fireflies::update_and_render(commands, s, input, speaker);
        }
        Splat(s) => {
            splat::update_and_render(commands, s, input, speaker);
        }
    }
}
