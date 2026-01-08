mod python;

use std::{fmt, iter::{empty, once, repeat}, ops::{BitAndAssign, BitOr}, sync::LazyLock};

use arrayvec::ArrayVec;
use itertools::Itertools;
use num_traits::PrimInt;
use unicode_width::UnicodeWidthStr;

// 立っているビットの位置のイテレーターを取得します。

fn bits<T: PrimInt + BitAndAssign>(x: T) -> impl Iterator<Item = usize> + Clone {
    let mut x = x;

    std::iter::from_fn(move || {
        if x == T::zero() {
            return None
        }

        let result = x.trailing_zeros() as usize;

        x &= x - T::one(); // pop LSB（Least Significant Bit）

        Some(result)
    })
}

// 駒の移動先を表す配列です。

static NEXTS: LazyLock<[[u16; 4 * 3]; 5]> = LazyLock::new(|| {  // [Piece、ビットの位置]
    let n = |bit_board| (bit_board & 0b_000_111_111_111) << 3;
    let s = |bit_board| (bit_board & 0b_111_111_111_000) >> 3;
    let e = |bit_board| (bit_board & 0b_011_011_011_011) << 1;
    let w = |bit_board| (bit_board & 0b_110_110_110_110) >> 1;

    (0..5).into_iter()
        .map(|piece_bit| {
            (0..4 * 3).into_iter()
                .map(|bit_board_bit| (1 << bit_board_bit) as u16)
                .map(|bit_board| match piece_bit {
                    0 /* ひよこ   */ => n(bit_board),
                    1 /* きりん   */ => n(bit_board) | s(bit_board) | e(bit_board) | w(bit_board),
                    2 /* ぞう     */ =>                                                             n(e(bit_board)) | n(w(bit_board)) | s(e(bit_board)) | s(w(bit_board)),
                    3 /* ライオン */ => n(bit_board) | s(bit_board) | e(bit_board) | w(bit_board) | n(e(bit_board)) | n(w(bit_board)) | s(e(bit_board)) | s(w(bit_board)),
                    4 /* にわとり */ => n(bit_board) | s(bit_board) | e(bit_board) | w(bit_board) | n(e(bit_board)) | n(w(bit_board)),
                    _                => unreachable!()
                })
                .collect_array()
                .unwrap()
        })
        .collect_array()
        .unwrap()
});

// ゲームの状態です。

#[derive(Clone, Copy)]
pub struct State {
    pieces: [u8; 8],       // 駒（先手由来×4 + 後手由来×4）
    ownership: u8,         // 駒を所有しているか
    bit_boards: [u16; 8],  // 駒単位の盤面（持ち駒は、対応するbit_boardが0になります）
    turn: u16              // ターン（0, 1, 2, ...）
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state_string = |index: usize| {
            let string = bits(self.pieces[index])
                .map(|piece_bit| {
                    match piece_bit {
                        0 => "ひ",
                        1 => "き",
                        2 => "ぞ",
                        3 => "ラ",
                        4 => "に",
                        _ => unreachable!()
                    }
                })
                .join("|");

            format!("{}{}{}", string, if (0..4).contains(&index) { "△" } else { "▽" }, " ".repeat(11 - string.width()))
        };

        let (ownership, bit_boards) = (|| {
            if self.turn % 2 == 0 {
                return (self.ownership, self.bit_boards);
            }

            (!self.ownership, self.bit_boards.map(|bit_board| bit_board.reverse_bits() >> 4))
        })();

        write!(
            f,
            "{}",
            empty()
                .chain(
                    bits(!ownership)
                        .filter(|index| bit_boards[*index] == 0)
                        .map(state_string)
                )
                .chain(once("-".repeat((2 + 11 + 2) * 3 + 2 * 2).to_string()))
                .chain(
                    (0..4).into_iter()
                        .map(|r| {
                            (0..3).into_iter()
                                .map(|c| {
                                    let Some(index) = bit_boards.iter().position(|bit_board| bit_board & 1 << (r * 3 + c) != 0) else {
                                        return " ".repeat(2 + 11 + 2).to_string();
                                    };

                                    format!("{}{}", if ownership & 1 << index != 0 { "▲" } else { "▼" }, state_string(index))
                                })
                                .rev()
                                .join("　")
                        })
                        .rev()
                )
                .chain(once("-".repeat((2 + 11 + 2) * 3 + 2 * 2).to_string()))
                .chain(
                    bits(ownership)
                        .filter(|index| bit_boards[*index] == 0)
                        .map(state_string)
                )
                .join("\n")
        )
    }
}

// ゲームのルールです。

pub struct Game;

impl Game {
    // コンストラクタ。

    pub fn initial_state() -> State {
        State {
            pieces:     [0b_0_1111, 0b_0_1111, 0b_0_1111, 0b_0_1111, 0b_0_1111, 0b_0_1111, 0b_0_1111, 0b_0_1111],
            ownership:  0b_0000_1111,
            bit_boards: [0b_000_000_000_001, 0b_000_000_000_010, 0b_000_000_000_100, 0b_000_000_010_000, 0b_000_010_000_000, 0b_001_000_000_000, 0b_010_000_000_000, 0b_100_000_000_000],
            turn:       0
        }
    }

    // 合法手の集合を取得します。

    pub fn legal_actions(state: &State) -> impl Iterator<Item = (u8, u8)> {
        // 自分の駒のBitBoardを取得します。

        let ally_bit_board = bits(state.ownership)
            .filter(|index| state.bit_boards[*index] != 0)
            .map(|index| state.bit_boards[index])
            .fold(0, BitOr::bitor);

        // 敵の駒のBitBoardを取得します。

        let enemy_bit_board = bits(!state.ownership)
            .filter(|index| state.bit_boards[*index] != 0)
            .map(|index| state.bit_boards[index])
            .fold(0, BitOr::bitor);

        // 駒を指すアクションを取得します。

        let move_piece_actions = bits(state.ownership)
            .filter(|index| state.bit_boards[*index] != 0)
            .flat_map(move |index| {
                let prev_bit = state.bit_boards[index].trailing_zeros() as u8;
                let next_bits = {
                    bits(
                        bits(state.pieces[index])
                            .map(|piece_bit| NEXTS[piece_bit][prev_bit as usize] & !ally_bit_board)
                            .fold(0, BitOr::bitor)
                    ).into_iter().map(|bit| bit as u8)
                };

                repeat(prev_bit).zip(next_bits)
            });

        // 持ち駒を打つアクションを取得します。

        let put_hand_actions = {
            let number_of_hands = bits(state.ownership)
                .filter(|index| state.bit_boards[*index] == 0)
                .count();

            let prev_bits = (4 * 3..4 * 3 + number_of_hands).into_iter().map(|bit| bit as u8);
            let next_bits = bits(!(ally_bit_board | enemy_bit_board) & 0b_111_111_111_111).map(|bit| bit as u8);  // どうぶつしょうぎには「行き所のない駒」ルールはありません（「ひよこ」を最上段に打てます）。

            prev_bits.flat_map(move |prev_bit| repeat(prev_bit).zip(next_bits.clone()))
        };

        // アクションの集合をリターンします。

        move_piece_actions.chain(put_hand_actions)
    }

    // 「使い切り」による収束（収縮？）を実施します。

    fn collapse(state: &State) -> State {
        let mut result = state.clone();

        // 先手由来と後手由来の2つで収束（収縮？）を実施します（本当は片方だけでよいはずだけど……）。

        for begin_index in [0, 4] {
            'outer: loop {
                // 成っている駒を元に戻します。

                let pieces = result.pieces[begin_index..begin_index + 4].iter().map(|piece| (piece | piece >> 4) & 0b_0000_1111).collect_array::<4>().unwrap();

                // 駒の可能性の組み合わせすべてで、ループを回します。

                for target_piece in 0b_0001..0b_1111 {
                    // 組み合わせ通りの駒の数を数えます。

                    let number_of_pieces = pieces.iter()
                        .filter(|piece| **piece == target_piece)
                        .count() as u32;

                    // 組み合わせ通り駒の数が、その可能性を満たす駒の数（どうぶつしょうぎは4種類4駒なので立っているビットの数と同じ）より小さいなら、他の駒にまだ可能性があるのでコンティニューします。

                    if number_of_pieces < target_piece.count_ones() {
                        continue;
                    }

                    // 可能性を削除する駒の集合を作成します。

                    let mut iter = pieces.iter()
                        .enumerate()
                        .filter(|(_, piece)| **piece != target_piece && **piece & target_piece != 0)
                        .peekable();

                    // 集合が空なら状態が変わらないので、コンティニューします。

                    if iter.peek().is_none() {
                        continue;
                    }

                    // 駒の可能性を削除します。

                    for (index, _) in iter {
                        result.pieces[begin_index + index] &= !(target_piece | target_piece << 4);
                    }

                    // 駒の状態が変わったので最初からやり直します。

                    continue 'outer;
                }

                // やり直しせずに最後まできたので、終了（無限ループを脱出）します。

                break;
            }
        }

        result
    }

    // 次のステートを取得します。

    pub fn next_state(state: &State, action: (u8, u8)) -> Option<State> {
        let mut result = state.clone();

        // 合法手であることをチェックします。

        if !Game::legal_actions(&state).contains(&action) {
            return None;
        }

        // 次のステートを取得します。

        if action.0 < 4 * 3 {
            // 駒を指すアクションを実行します。

            (|| {
                // 移動先に駒があれば、取って持ち駒に加えます。

                if let Some(index) = result.bit_boards.iter().position(|bit_board| bit_board & 1 << action.1 != 0) {
                    result.pieces[index] = {
                        let mut result = result.pieces[index];

                        // 成っている駒を元に戻します。

                        result = (result | result >> 4) & 0b_0000_1111;

                        result
                    };
                    result.ownership |= 1 << index;
                    result.bit_boards[index] = 0;
                }

                // 移動する駒を取得します。

                let index = result.bit_boards.iter().position(|bit_board| bit_board & 1 << action.0 != 0).unwrap();

                // 「絞り込み」による収束（収縮？）を実施します。

                result.pieces[index] = bits(result.pieces[index])
                    .map(|piece| if NEXTS[piece][action.0 as usize] & 1 << action.1 != 0 { 1 << piece } else { 0 })
                    .fold(0, BitOr::bitor);

                // 駒の状態が変更されたので、「使い切り」による収束（収縮？）を実施します。

                result = Game::collapse(&result);

                // 駒を移動します。

                result.pieces[index] = {
                    let mut result = result.pieces[index];

                    // 敵のエリアに移動したなら、駒を成らせします。

                    if (9..12).contains(&action.1) && result & 0b_0000_0001 != 0 {
                        result = (result | result << 4) & 0b_0001_1110;  // どうぶつしょうぎで成るのは「ひよこ」だけ。
                    }

                    result
                };
                result.bit_boards[index] = 1 << action.1;

                // 持ち駒から、ライオンの可能性を外します。

                for index in bits(result.ownership)
                    .filter(|index| result.bit_boards[*index] == 0 && result.pieces[*index] & 0b_0000_1000 != 0 && result.pieces[*index] != 0b_0000_1000)
                    .collect::<ArrayVec<_, 8>>()
                {
                    result.pieces[index] &= !0b_0000_1000;
                }

                // 持ち駒の状態が変更されたので、「使い切り」による収束（収縮？）を実施します。

                result = Game::collapse(&result);
            })();
        } else {
            // 持ち駒を打つアクションを実行します。

            (|| {
                // 打つ駒を取得します。

                let index = bits(result.ownership)
                    .filter(|index| result.bit_boards[*index] == 0)
                    .nth(action.0 as usize - 4 * 3)
                    .unwrap();

                // 駒を打ちます。

                result.bit_boards[index] = 1 << action.1;

                // どうぶつしょうぎには「行き所のない駒」ルールがないので、持ち駒を打つときには収束（収縮？）は発生しません。
            })();
        }

        // 盤面を回転します。

        result.ownership = !result.ownership;
        result.bit_boards = result.bit_boards.map(|bit_board| bit_board.reverse_bits() >> 4);

        // ターンを進めます。

        result.turn += 1;

        // 次のステートを返します。

        Some(result)
    }

    // 勝利したかを取得します。

    fn won(state: &State) -> bool {
        bits(!state.ownership).any(|index| state.bit_boards[index] == 0 && state.pieces[index] == 0b_0000_1000)  // next_stateで盤面が回転しているので、!state.ownership。
    }
}
