use std::{collections::{BTreeSet, VecDeque}, iter::once, ops::BitOr};

use itertools::Itertools;
use quantum_animal_shogi_core::{Game, State, bits};


// 以下を参照して作成しました。
//
// [「どうぶつしょうぎ」の完全解析](https://www.tanaka.ecc.u-tokyo.ac.jp/ktanaka/dobutsushogi/animal-private.pdf)


// 状態をu128に変換します。

fn convert_state_to_u128(state: &State) -> u128 {
    let pieces: [u128; 8] = (0..8)
        .into_iter()
        .map(|i| state.pieces[i] as u128 | if state.pieces[i].count_ones() == 1 || i < 4 { 0 } else { 1 } << 5 | if state.ownership & 1 << i != 0 { 0 } else { 1 } << 6)
        .collect_array()
        .unwrap();

    let result_0 = (0..8)
        .into_iter()
        .filter(|i| state.bit_boards[*i] != 0)
        .map(|i| pieces[i] << 7 * state.bit_boards[i].trailing_zeros())
        .fold(0, BitOr::bitor);

    let result_1 = (0..8)
        .into_iter()
        .filter(|i| state.bit_boards[*i] == 0)
        .map(|i| pieces[i])
        .sorted()
        .enumerate()
        .fold(0, |acc, (i, piece)| acc | piece << 7 * i);

    result_0 | result_1 << 84
}

// 盤面を左右反転します。

fn turn_left_right(state: &State) -> State {
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

// 終端局面か判断します。

fn is_terminal_state(state: &State) -> bool {
    // // すべての駒が確定しているなら、「どうぶつしょうぎ」の強解決結果に含まれるならそれを活用できるし、そうでなくてもu64で状態を表して別途探索すれば良いので、とりあえずの終端局面とします。

    // if state.pieces.iter().all(|piece| piece.count_ones() == 1) {
    //     return true;
    // }

    // 敵のライオンを取れるなら勝ち確定局面とします。

    for action in Game::legal_actions(&state) {
        let next_state = Game::next_state(&state, action);

        if bits(!next_state.ownership).any(|index| next_state.bit_boards[index] == 0 && next_state.pieces[index] == 0b_0000_1000) {
            return true;
        }
    }

    // 勝ち確定局面ではない場合で、敵のライオンの可能性を持つ駒が自陣にいれば負け確定局面とします。

    if bits(!state.ownership).any(|index| state.bit_boards[index] & 0b_000_000_000_111 != 0 && state.pieces[index] & 0b_0000_1000 != 0) {
        return true;
    }

    // どちらでもなければ、終端局面ではありません。

    false
}

// メイン・ルーチンです。

fn main() {
    let (mut stack, mut visited) = {
        // りょうしどうぶつしょうぎの初期状態を設定します。

        let state = Game::initial_state();

        // 「どうぶつしょうぎ」の初期状態を設定します。この場合で「どうぶつしょうぎ」の完全解析と同じ結果になるなら、処理は概ね正しいはず。 ← 12分で探索が終了し、246,803,167で一致した！

        // let state = State {
        //     pieces:     [0b_0_0010, 0b_0_1000, 0b_0_0100, 0b_0_0001, 0b_0_0001, 0b_0_0100, 0b_0_1000, 0b_0_0010],
        //     ownership:  0b_0000_1111,
        //     bit_boards: [0b_000_000_000_001, 0b_000_000_000_010, 0b_000_000_000_100, 0b_000_000_010_000, 0b_000_010_000_000, 0b_001_000_000_000, 0b_010_000_000_000, 0b_100_000_000_000]
        // };

        let result_0 = once(state).collect::<VecDeque<_>>();
        let result_1 = result_0.clone().iter().map(convert_state_to_u128).collect::<BTreeSet<_>>();

        (result_0, result_1)
    };

    while let Some(state) = stack.pop_back() {
        if is_terminal_state(&state) {
            continue;
        }

        for action in Game::legal_actions(&state) {
            let next_state = Game::next_state(&state, action);

            let next_state_u128 = [convert_state_to_u128(&next_state), convert_state_to_u128(&turn_left_right(&next_state))].into_iter().min().unwrap();

            if visited.contains(&next_state_u128) {
                continue;
            }

            visited.insert(next_state_u128);
            stack.push_back(next_state);

            println!("{}", visited.len());
        }
    }

    println!("{}", visited.len());
}
