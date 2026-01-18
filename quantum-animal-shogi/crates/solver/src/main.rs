use std::{collections::{HashSet, VecDeque}, iter::once};

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

// fn u128_to_state(state: &u128) -> State {
//     let iter = (0..8)
//         .into_iter()
//         .map(|i| (state >> (i * 11)) as u16 & 0b_0000_0111_1111_1111);

//     let pieces = iter
//         .clone()
//         .map(|piece_state| (piece_state & 0b_0000_0000_0001_1111) as u8)
//         .collect_array()
//         .unwrap();

//     let ownership = iter
//         .clone()
//         .enumerate()
//         .map(|(i, piece_state)| (if piece_state >> 6 & 0b_0000_0000_0000_0001 != 0 { 1 } else { 0 }) << i)
//         .fold(0, BitOr::bitor);

//     let bit_boards = iter
//         .map(|piece_state| {
//             let position = piece_state >> (6 + 1) & 0b_0000_0000_0000_1111;

//             if position == 4 * 3 { 0 } else { 1 << position }
//         })
//         .collect_array()
//         .unwrap();

//     State {
//         pieces,
//         ownership,
//         bit_boards
//     }
// }

fn left_right_turned(state: &State) -> State {
    let mut result = *state;

    result.bit_boards = state.bit_boards
        .map(|bit_board| {
            [(0, 2), (3, 5), (6, 8), (9, 11)]
                .into_iter()
                .fold(
                    bit_board,
                    |acc, (bit_x, bit_y)| {
                        let bit_x_value = if acc & 1 << bit_x != 0 { 1 } else { 0 };
                        let bit_y_value = if acc & 1 << bit_y != 0 { 1 } else { 0 };

                        (acc & !(1 << bit_x) | bit_y_value << bit_x) & !(1 << bit_y) | bit_x_value << bit_y
                    }
                )
        });

    result
}

fn main() {
    let (mut deque, mut states) = {
        let result_0 = once(Game::initial_state()).collect::<VecDeque<_>>();
        let result_1 = result_0.clone().iter().map(state_to_u128).collect::<HashSet<_>>();

        (result_0, result_1)
    };

    while let Some(state) = deque.pop_back() {
        if Game::won(&state) || Game::lost(&state) {
            println!("{}", state.to_string());
            println!();

            continue;
        }

        for action in Game::legal_actions(&state) {
            let next_state = Game::next_state(&state, action).unwrap();
            let next_state_u128 = [state_to_u128(&next_state), state_to_u128(&left_right_turned(&state))].into_iter().min().unwrap();

            if states.contains(&next_state_u128) {
                continue;
            }

            states.insert(next_state_u128);
            deque.push_back(next_state);
        }
    }

    println!("{}", states.len());
}
