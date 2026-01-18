use std::ops::BitOr;

use itertools::Itertools;
use quantum_animal_shogi_core::{Game, State};

fn state_to_u128(state: &State) -> u128 {
    (0..8)
        .into_iter()
        .map(|i| {
            let mut result = 0_u16;

            result |= state.pieces[i] as u16;
            result |= (if state.ownership & 1 << i != 0 { 1 } else { 0 }) << 6;
            result |= (if state.bit_boards[i] == 0 { 4 * 3 } else { state.bit_boards[i].trailing_zeros() as u16 }) << (6 + 1);

            (i, result)
        })
        .fold(
            0,
            |acc, (i, piece_state)| acc | (piece_state as u128) << (i * 11)
        )
}

fn u128_to_state(state: &u128) -> State {
    let iter = (0..8)
        .into_iter()
        .map(|i| (state >> (i * 11)) as u16 & 0b_0000_0111_1111_1111);

    let pieces = iter
        .clone()
        .map(|piece_state| (piece_state & 0b_0000_0000_0001_1111) as u8)
        .collect_array()
        .unwrap();

    let ownership = iter
        .clone()
        .enumerate()
        .map(|(i, piece_state)| (if piece_state >> 6 & 0b_0000_0000_0000_0001 != 0 { 1 } else { 0 }) << i)
        .fold(0, BitOr::bitor);

    let bit_boards = iter
        .map(|piece_state| {
            let position = piece_state >> (6 + 1) & 0b_0000_0000_0000_1111;

            if position == 4 * 3 { 0 } else { 1 << position }
        })
        .collect_array()
        .unwrap();

    State {
        pieces,
        ownership,
        bit_boards,
        turn: 0
    }
}

fn main() {
    let mut state = Game::initial_state();
    let mut turn = 0;

    while !Game::won(&state) && !Game::lost(&state) {
        if turn % 2 == 0 {
            println!("{}", state.to_string());
            println!();
            println!("{}", u128_to_state(&state_to_u128(&state)).to_string());
            println!();
            println!();
        }

        state = {
            let mut actions = Game::legal_actions(&state);
            actions.next();
            actions.next();
            actions.next();

            let action = actions.next().unwrap();

            Game::next_state(&state, action).unwrap()
        };
        turn += 1;
    }
}
