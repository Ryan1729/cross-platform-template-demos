use common::*;
use game_state::{
    get_card_offset, get_card_position, Choice, Chosen, GameState, Hand, LogHeading, Spread,
};
use platform_types::{Button, Input, Speaker, State, StateParams, SFX};
use rand::Rng;

#[allow(dead_code)]
enum Face {
    Up,
    Down,
}

fn draw_hand_ltr(
    framebuffer: &mut Framebuffer,
    hand: &Hand,
    offset: u8,
    (mut x, y): (u8, u8),
    face: Face,
) {
    match face {
        Face::Up => {
            for &card in hand.iter() {
                framebuffer.draw_card(card, x, y);

                x += offset;
            }
        }
        Face::Down => {
            for &_card in hand.iter() {
                framebuffer.draw_card_back(x, y);

                x += offset;
            }
        }
    }
}

fn draw_hand_ttb(
    framebuffer: &mut Framebuffer,
    hand: &Hand,
    offset: u8,
    (x, mut y): (u8, u8),
    face: Face,
) {
    match face {
        Face::Up => {
            for &card in hand.iter() {
                framebuffer.draw_card(card, x, y);

                y += offset;
            }
        }
        Face::Down => {
            for &_card in hand.iter() {
                framebuffer.draw_card_back(x, y);

                y += offset;
            }
        }
    }
}

fn draw_hand(framebuffer: &mut Framebuffer, hand: &Hand, face: Face) {
    let offset = get_card_offset(hand.spread, hand.len());

    match hand.spread {
        Spread::LTR((x, _), y) => {
            draw_hand_ltr(framebuffer, hand, offset, (x, y), face);
        }
        Spread::TTB((y, _), x) => {
            draw_hand_ttb(framebuffer, hand, offset, (x, y), face);
        }
    }
}

fn draw_hand_with_cursor_ltr(
    framebuffer: &mut Framebuffer,
    hand: &Hand,
    offset: u8,
    (mut x, y): (u8, u8),
    index: usize,
) {
    let mut selected_card_and_offset = None;
    for (i, &card) in hand.iter().enumerate() {
        if i == index {
            selected_card_and_offset = Some((card, x));
            x += offset;

            continue;
        }
        framebuffer.draw_card(card, x, y);

        x += offset;
    }

    if let Some((card, cursor_offset)) = selected_card_and_offset {
        framebuffer.draw_highlighted_card(card, cursor_offset, y);
    }
}

fn draw_hand_with_cursor_ttb(
    framebuffer: &mut Framebuffer,
    hand: &Hand,
    offset: u8,
    (x, mut y): (u8, u8),
    index: usize,
) {
    let mut selected_card_and_offset = None;
    for (i, &card) in hand.iter().enumerate() {
        if i == index {
            selected_card_and_offset = Some((card, y));
            y += offset;

            continue;
        }
        framebuffer.draw_card(card, x, y);

        y += offset;
    }

    if let Some((card, cursor_offset)) = selected_card_and_offset {
        framebuffer.draw_highlighted_card(card, x, cursor_offset);
    }
}

fn draw_hand_with_cursor(framebuffer: &mut Framebuffer, hand: &Hand, index: usize) {
    let offset = get_card_offset(hand.spread, hand.len());

    match hand.spread {
        Spread::LTR((x, _), y) => {
            draw_hand_with_cursor_ltr(framebuffer, hand, offset, (x, y), index);
        }
        Spread::TTB((y, _), x) => {
            draw_hand_with_cursor_ttb(framebuffer, hand, offset, (x, y), index);
        }
    }
}

fn draw_event_log(framebuffer: &mut Framebuffer, state: &GameState) {
    framebuffer.bottom_six_slice(WINDOW_TOP_LEFT, 0, 0, SCREEN_WIDTH as u8, state.log_height);

    let mut y = SPRITE_SIZE;
    for line in state.event_log.get_window_slice(state.log_top_index) {
        framebuffer.print_line(line, SPRITE_SIZE, y, WHITE_INDEX);

        y += FONT_SIZE;
        if y >= state.log_height {
            break;
        }
    }
}

fn move_cursor(state: &mut GameState, input: Input, speaker: &mut Speaker) -> bool {
    if input.pressed_this_frame(Button::Right) {
        if state.hand_index < state.hand.len().saturating_sub(1) {
            state.hand_index = state.hand_index.saturating_add(1);
        }
        speaker.request_sfx(SFX::CardSlide);
        true
    } else if input.pressed_this_frame(Button::Left) {
        state.hand_index = state.hand_index.saturating_sub(1);
        speaker.request_sfx(SFX::CardSlide);
        true
    } else {
        false
    }
}

fn is_wild(card: Card) -> bool {
    get_rank(card) == 8 - 1
}

fn can_play(state: &GameState, &card: &Card) -> bool {
    if let Some(&top_of_discard) = state.discard.last() {
        is_wild(card) || if is_wild(top_of_discard) {
            state.top_wild_declared_as == Some(get_suit(card))
        } else {
            get_suit(top_of_discard) == get_suit(card) || get_rank(top_of_discard) == get_rank(card)
        }
    } else {
        true
    }
}

//Since this uses rng, callling this in response to repeatable user input allows rng manipulation.
fn cpu_would_play(state: &mut GameState, playerId: PlayerID) -> Option<u8> {
    let playable: Vec<(usize, Card)> = {
        let hand = state.get_hand(playerId);
        hand.iter()
            .cloned()
            .enumerate()
            .filter(|(_, card)| can_play(state, card))
            .collect()
    };

    state.rng.choose(&playable).map(|&(i, _)| i as u8)
}

fn choose_suit(state: &mut GameState) -> Option<Suit> {
    match state.choice {
        Choice::NoChoice => {
            state.choice = Choice::OfSuit;
            None
        }
        Choice::Already(Chosen::Suit(suit)) => {
            state.choice = Choice::NoChoice;
            Some(suit)
        }
        _ => None,
    }
}
fn choose_play_again(state: &mut GameState) -> Option<()> {
    match state.choice {
        Choice::NoChoice => {
            state.choice = Choice::OfUnit;
            None
        }
        Choice::Already(Chosen::Unit(unit)) => {
            state.choice = Choice::NoChoice;
            Some(unit)
        }
        _ => None,
    }
}

fn move_to_discard(state: &mut GameState, card: Card) {
    if !is_wild(card) {
        state.top_wild_declared_as = None;
    }

    state.discard.push(card);
}

fn log_wild_selection(state: &mut GameState, player: PlayerID) {
    if let Some(suit) = state.top_wild_declared_as {
        let player_name = state.player_name(player);
        let suit_str = get_suit_str(suit);
        let event_str = &[
            player_name.as_bytes(),
            b" selected ",
            suit_str.as_bytes(),
            b".",
        ]
            .concat();
        state.event_log.push(event_str)
    }
}

fn advance_card_animations(state: &mut GameState) {
    // I should really be able to use `Vec::retain` here,
    // but that passes a `&T` insteead of a `&mut T`.

    let mut i = state.card_animations.len() - 1;
    loop {
        let (is_complete, last_pos) = {
            let animation = &mut state.card_animations[i];

            let last_pos = (animation.card.x, animation.card.y);

            animation.approach_target();

            (animation.is_complete(), last_pos)
        };

        if is_complete {
            let mut animation = state.card_animations.remove(i);

            let card = animation.card.card;

            match animation.completion_action {
                Action::MoveToDiscard => {
                    move_to_discard(state, card);
                }
                Action::SelectWild(playerId) => {
                    if is_cpu_player(&state, playerId) {
                        state.top_wild_declared_as = {
                            let hand = state.get_hand(playerId);
                            hand.most_common_suit()
                        };
                        log_wild_selection(state, playerId);
                        move_to_discard(state, card);
                    } else {
                        if let Some(suit) = choose_suit(state) {
                            state.top_wild_declared_as = Some(suit);
                            log_wild_selection(state, playerId);
                            move_to_discard(state, card);
                        } else {
                            //wait until they choose
                            animation.card.x = last_pos.0;
                            animation.card.y = last_pos.1;
                            state.card_animations.push(animation);
                        }
                    }
                }
                Action::MoveToHand(playerId) => {
                    state.get_hand_mut(playerId).push(card);
                }
            }
        }

        if i == 0 {
            break;
        }
        i -= 1;
    }
}

fn get_discard_animation(
    state: &mut GameState,
    player: PlayerID,
    card_index: u8,
) -> Option<CardAnimation> {
    state
        .remove_positioned_card(player, card_index)
        .map(|card| {
            let player_name = state.player_name(player);

            let card_string = get_card_string(card.card);

            let rank = get_rank(card.card);

            let event_str = &[
                player_name.as_bytes(),
                if rank == 0 || rank == 7 {
                    b" played an "
                } else {
                    b" played a "
                },
                card_string.as_bytes(),
                b".",
            ]
                .concat();

            state.event_log.push(event_str);

            if is_wild(card.card) {
                CardAnimation::new(card, DISCARD_X, DISCARD_Y, Action::SelectWild(player))
            } else {
                CardAnimation::new(card, DISCARD_X, DISCARD_Y, Action::MoveToDiscard)
            }
        })
}

fn get_draw_animation(state: &mut GameState, player: PlayerID) -> Option<CardAnimation> {
    let (spread, len) = {
        let hand = state.get_hand(player);

        (hand.spread, hand.len())
    };
    let card = {
        if let Some(c) = state.deck.draw() {
            Some(c)
        } else {
            let top_card = state.discard.draw()?;

            state.deck.fill(state.discard.drain());
            state.deck.shuffle(&mut state.rng);

            state.discard.push(top_card);

            state.deck.draw()
        }
    }?;

    let (x, y) = get_card_position(spread, len + 1, len);

    let player_name = state.player_name(player);

    let event_str = &[player_name.as_bytes(), b" drew a card."].concat();

    state.event_log.push(event_str);

    Some(CardAnimation::new(
        PositionedCard {
            card,
            x: DECK_X,
            y: DECK_Y,
        },
        x,
        y,
        Action::MoveToHand(player),
    ))
}

#[inline]
fn push_if<T>(vec: &mut Vec<T>, op: Option<T>) {
    if let Some(t) = op {
        vec.push(t);
    }
}

fn is_cpu_player(state: &GameState, playerId: PlayerID) -> bool {
    (playerId as usize) < state.cpu_hands.len()
}

fn take_turn(state: &mut GameState, input: Input, speaker: &mut Speaker) {
    let player = state.current_player;
    match player {
        p if is_cpu_player(&state, p) => {
            if let Some(index) = cpu_would_play(state, p) {
                let animation = get_discard_animation(state, player, index);
                push_if(&mut state.card_animations, animation);
            } else {
                let animation = get_draw_animation(state, player);
                push_if(&mut state.card_animations, animation);
            }

            state.current_player += 1;
        }
        _ => {
            if move_cursor(state, input, speaker) {
                //Already handled.
            } else if input.pressed_this_frame(Button::A) {
                let index = state.hand_index;

                let can_play_it = {
                    state
                        .hand
                        .get(index)
                        .map(|card| can_play(&state, card))
                        .unwrap_or(false)
                };

                if can_play_it {
                    let animation = get_discard_animation(state, player, index);

                    push_if(&mut state.card_animations, animation);

                    state.current_player = 0;
                } else {
                    //TODO good feedback. Tint the card red or shake it or something?
                }
            } else if input.pressed_this_frame(Button::B) {
                let animation = get_draw_animation(state, player);
                push_if(&mut state.card_animations, animation);

                state.current_player = 0;
            }
        }
    }

    let player_ids: Vec<PlayerID> = state.player_ids();

    let winners: Vec<PlayerID> = player_ids
        .iter()
        .filter(|&&player| state.get_hand(player).len() == 0)
        .cloned()
        .collect();

    if winners.len() > 0 {
        state.winners = winners;
    }
}

fn update(state: &mut GameState, input: Input, speaker: &mut Speaker) {
    match state.log_heading {
        LogHeading::Up => {
            state.log_height = state.log_height.saturating_sub(SPRITE_SIZE);
        }
        LogHeading::Down => {
            if state.log_height <= SCREEN_HEIGHT as u8 - SPRITE_SIZE {
                state.log_height += SPRITE_SIZE;
            }
        }
    }

    if input.pressed_this_frame(Button::Start) {
        state.log_heading = match state.log_heading {
            LogHeading::Up => LogHeading::Down,
            LogHeading::Down => LogHeading::Up,
        };
    }

    if state.log_height > 0 {
        if input.pressed_this_frame(Button::Up) {
            state.log_top_index = state.log_top_index.saturating_sub(1);
        //TODO feedback when you hit the top edge
        } else if input.pressed_this_frame(Button::Down) {
            if state.log_top_index < state.event_log.len() {
                state.log_top_index += 1;
            } else {
                //TODO feedback when you hit the bottom edge
            }
        }
    } else if state.choice.is_idle() {
        if state.card_animations.len() == 0 {
            if state.winners.len() == 0 {
                take_turn(state, input, speaker);
            }
        } else {
            advance_card_animations(state);

            move_cursor(state, input, speaker);
        }
    }
}

//calling this once will swallow multiple presses on the button. We could either
//pass in and return the number of presses to fix that, or this could simply be
//called multiple times per frame (once for each click).
fn do_button(
    framebuffer: &mut Framebuffer,
    context: &mut UIContext,
    input: Input,
    speaker: &mut Speaker,
    spec: &ButtonSpec,
) -> bool {
    let mut result = false;

    let id = spec.id;

    if context.active == id {
        if input.released_this_frame(Button::A) {
            result = context.hot == id;

            context.set_not_active();
        }
        context.set_next_hot(id);
    } else if context.hot == id {
        if input.pressed_this_frame(Button::A) {
            context.set_active(id);
            speaker.request_sfx(SFX::ButtonPress);
        }
        context.set_next_hot(id);
    }

    if context.active == id && input.gamepad.contains(Button::A) {
        framebuffer.button_pressed(spec.x, spec.y, spec.w, spec.h);
    } else if context.hot == id {
        framebuffer.button_hot(spec.x, spec.y, spec.w, spec.h);
    } else {
        framebuffer.button(spec.x, spec.y, spec.w, spec.h);
    }

    let text = spec.text.as_bytes();

    let (x, y) = center_line_in_rect(text.len() as u8, ((spec.x, spec.y), (spec.w, spec.h)));

    //Long labels aren't great UX anyway, I think, so don't bother reflowing.
    //Add the extra bit to `y` because the current graphics looks better that way.
    framebuffer.print(spec.text.as_bytes(), x, y + (FONT_SIZE / 4), WHITE_INDEX);

    return result;
}

#[inline]
pub fn do_suit_choice(
    framebuffer: &mut Framebuffer,
    state: &mut GameState,
    input: Input,
    speaker: &mut Speaker,
) {
    framebuffer.full_window();
    {
        let text = b"choose a suit for the 8 to be";

        let (x, _) = center_line_in_rect(
            text.len() as u8,
            (
                (SPRITE_SIZE, SPRITE_SIZE),
                (NINE_SLICE_MAX_INTERIOR_SIZE, NINE_SLICE_MAX_INTERIOR_SIZE),
            ),
        );

        framebuffer.print(text, x, SPRITE_SIZE * 2, WHITE_INDEX);
    }

    let w = NINE_SLICE_MAX_INTERIOR_SIZE;
    let h = SPRITE_SIZE * 3;
    let x = SPRITE_SIZE;

    for (i, suit) in Suits::ALL.iter().cloned().enumerate() {
        let i = (i + 1) as u8;

        let (_, suit_char) = get_suit_colour_and_char(suit);

        let mut text = String::with_capacity(1);
        text.push(char::from(suit_char));

        let spec = ButtonSpec {
            x,
            y: h * i,
            w,
            h,
            id: i,
            text,
        };

        if do_button(framebuffer, &mut state.context, input, speaker, &spec) {
            state.choice = Choice::Already(Chosen::Suit(suit));
        }
    }

    if state.context.hot == 0 || state.context.hot > 4 {
        state.context.set_next_hot(1);
    } else if input.pressed_this_frame(Button::Up) {
        let next = dice_mod(state.context.hot - 1, 4);
        state.context.set_next_hot(next);
    } else if input.pressed_this_frame(Button::Down) {
        let next = dice_mod(state.context.hot + 1, 4);
        state.context.set_next_hot(next);
    }
}

fn dice_mod(x: u8, m: u8) -> u8 {
    if x == 0 {
        m
    } else {
        (x.saturating_sub(1) % m).saturating_add(1)
    }
}

#[inline]
pub fn do_bool_choice(
    framebuffer: &mut Framebuffer,
    state: &mut GameState,
    input: Input,
    speaker: &mut Speaker,
) {
    framebuffer.full_window();

    {
        let question = b"Close this window?";

        let (x, y) = center_line_in_rect(
            question.len() as u8,
            (
                (SPRITE_SIZE, SPRITE_SIZE),
                (NINE_SLICE_MAX_INTERIOR_SIZE, NINE_SLICE_MAX_INTERIOR_SIZE),
            ),
        );

        framebuffer.print(question, x, y, WHITE_INDEX);
    }

    let w = SPRITE_SIZE * 5;
    let h = SPRITE_SIZE * 3;
    let y = SCREEN_HEIGHT as u8 - (h + SPRITE_SIZE);

    let spec1 = ButtonSpec {
        x: SPRITE_SIZE,
        y,
        w,
        h,
        id: 1,
        text: "yes".to_owned(),
    };

    if do_button(framebuffer, &mut state.context, input, speaker, &spec1) {
        state.choice = Choice::Already(Chosen::Bool(true));
    }

    let spec2 = ButtonSpec {
        x: SCREEN_WIDTH as u8 - (w + SPRITE_SIZE),
        y,
        w,
        h,
        id: 2,
        text: "no".to_owned(),
    };

    if do_button(framebuffer, &mut state.context, input, speaker, &spec2) {
        state.choice = Choice::Already(Chosen::Bool(false));
    }

    if state.context.hot != 1 && state.context.hot != 2 {
        state.context.set_next_hot(1);
    } else if input.pressed_this_frame(Button::Left) || input.pressed_this_frame(Button::Right) {
        if state.context.hot == 1 {
            state.context.set_next_hot(2);
        } else {
            state.context.set_next_hot(1);
        }
    }
}

#[inline]
pub fn do_unit_choice(
    framebuffer: &mut Framebuffer,
    state: &mut GameState,
    input: Input,
    speaker: &mut Speaker,
) {
    framebuffer.full_window();

    {
        let winner_text = reflow(
            &state.get_winner_text(),
            NINE_SLICE_MAX_INTERIOR_WIDTH_IN_CHARS as usize,
        );

        let dimensions = get_text_dimensions(winner_text.as_bytes());

        let (x, _) = center_rect_in_rect(
            dimensions,
            (
                (SPRITE_SIZE, SPRITE_SIZE),
                (NINE_SLICE_MAX_INTERIOR_SIZE, NINE_SLICE_MAX_INTERIOR_SIZE),
            ),
        );

        framebuffer.print(winner_text.as_bytes(), x, SPRITE_SIZE, 6);
    }

    {
        let question = b"would you like to play again?";

        let (x, y) = center_line_in_rect(
            question.len() as u8,
            (
                (SPRITE_SIZE, SPRITE_SIZE),
                (NINE_SLICE_MAX_INTERIOR_SIZE, NINE_SLICE_MAX_INTERIOR_SIZE),
            ),
        );

        framebuffer.print(question, x, y, WHITE_INDEX);
    }

    let w = SPRITE_SIZE * 5;
    let h = SPRITE_SIZE * 3;
    let y = SCREEN_HEIGHT as u8 - (h + SPRITE_SIZE);

    let (x, _) = center_rect_in_rect((w, h), ((0, y), (SCREEN_WIDTH as u8, h)));

    let text = "yes".to_owned();

    let spec1 = ButtonSpec {
        x,
        y,
        w,
        h,
        id: 1,
        text,
    };

    if do_button(framebuffer, &mut state.context, input, speaker, &spec1) {
        state.choice = Choice::Already(Chosen::Unit(()));
    }

    if state.context.hot != 1 {
        state.context.set_next_hot(1);
    }
}

#[inline]
pub fn update_and_render(
    framebuffer: &mut Framebuffer,
    state: &mut GameState,
    input: Input,
    speaker: &mut Speaker,
) {
    state.context.frame_init();

    update(state, input, speaker);

    invariant_assert_eq!(state.missing_cards(), vec![0; 0]);

    framebuffer.clearTo(GREEN);

    for hand in state.cpu_hands.iter() {
        draw_hand(framebuffer, hand, Face::Down);
    }

    framebuffer.draw_card_back(DECK_X, DECK_Y);

    match state.top_wild_declared_as {
        Some(suit) => {
            let (colour, suit_char) = get_suit_colour_and_char(suit);

            framebuffer.print_char(
                suit_char,
                DECK_X + card::WIDTH + 2,
                DECK_Y + (card::HEIGHT - FONT_SIZE) / 2,
                colour,
            );
        }
        None => {}
    }

    state
        .discard
        .iter()
        .last()
        .map(|&c| framebuffer.draw_card(c, DISCARD_X, DISCARD_Y));

    draw_hand_with_cursor(framebuffer, &state.hand, state.hand_index as usize);

    for &CardAnimation {
        card,
        completion_action,
        ..
    } in state.card_animations.iter()
    {
        match completion_action {
            Action::MoveToHand(_) => framebuffer.draw_card_back(card.x, card.y),
            Action::MoveToDiscard | Action::SelectWild(_) => {
                framebuffer.draw_card(card.card, card.x, card.y)
            }
        }
    }

    if state.winners.len() > 0 && state.card_animations.len() == 0 {
        if let Some(()) = choose_play_again(state) {
            state.reset();
        }
    }

    if state.log_height > 0 {
        draw_event_log(framebuffer, &state);
    } else {
        match state.choice {
            Choice::OfSuit => do_suit_choice(framebuffer, state, input, speaker),
            Choice::OfBool => do_bool_choice(framebuffer, state, input, speaker),
            Choice::OfUnit => do_unit_choice(framebuffer, state, input, speaker),
            _ => {}
        }
    }
}

pub struct BartogState {
    pub game_state: GameState,
    pub framebuffer: Framebuffer,
    pub input: Input,
    pub speaker: Speaker,
}

impl BartogState {
    pub fn new((seed, logger): StateParams) -> Self {
        let framebuffer = Framebuffer::new();

        BartogState {
            game_state: GameState::new(seed, logger),
            framebuffer,
            input: Input::new(),
            speaker: Speaker::new(),
        }
    }
}

impl State for BartogState {
    fn frame(&mut self, handle_sound: fn(SFX)) {
        update_and_render(
            &mut self.framebuffer,
            &mut self.game_state,
            self.input,
            &mut self.speaker,
        );

        self.input.previous_gamepad = self.input.gamepad;

        for request in self.speaker.drain() {
            handle_sound(request);
        }
    }

    fn press(&mut self, button: Button::Ty) {
        if self.input.previous_gamepad.contains(button) {
            //This is meant to pass along the key repeat, if any.
            //Not sure if rewriting history is the best way to do this.
            self.input.previous_gamepad.remove(button);
        }

        self.input.gamepad.insert(button);
    }

    fn release(&mut self, button: Button::Ty) {
        self.input.gamepad.remove(button);
    }

    fn get_frame_buffer(&self) -> &[u32] {
        &self.framebuffer.buffer
    }
}