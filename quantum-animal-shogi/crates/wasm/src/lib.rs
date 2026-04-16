use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use quantum_animal_shogi_core::{Game as Game_, State as State_};

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(js_namespace = console)]
//     fn log(s: &str);
// }

#[wasm_bindgen]
pub struct State {
    state: State_
}

#[wasm_bindgen]
impl State {
    #[wasm_bindgen(getter)]
    pub fn pieces(&self) -> Vec<u8> {
        self.state.pieces.into()
    }

    #[wasm_bindgen(getter)]
    pub fn ownership(&self) -> u8 {
        self.state.ownership
    }

    #[wasm_bindgen(getter, js_name = bitBoards)]
    pub fn bit_boards(&self) -> Vec<u16> {
        self.state.bit_boards.into()
    }
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Action(u8, u8);

impl From<(u8, u8)> for Action {
    fn from(action: (u8, u8)) -> Action {
        Action(action.0, action.1)
    }
}

#[wasm_bindgen(js_name = getInitialState)]
pub fn get_initial_state() -> State {
    State {
        state: Game_::initial_state()
    }
}

#[wasm_bindgen(js_name = getTurnedState)]
pub fn get_turned_state(state: &State) -> State {
    let mut result = state.state.clone();

    result.ownership = !result.ownership;
    result.bit_boards = result.bit_boards.map(|bit_board| bit_board.reverse_bits() >> 4);

    State { state: result }
}

#[wasm_bindgen(js_name = getLegalActions)]
pub fn get_legal_actions(state: &State) -> Vec<Action> {
    Game_::legal_actions(&state.state).map(|action| action.into()).collect()
}

#[wasm_bindgen(js_name = getNextState)]
pub fn get_next_state(state: &State, action: &Action) -> State {
    State { state: Game_::next_state(&state.state, (action.0, action.1)) }
}

#[wasm_bindgen]
pub fn won(state: &State) -> bool {
    Game_::won(&state.state)
}

#[wasm_bindgen]
pub fn lost(state: &State) -> bool {
    Game_::lost(&state.state)
}

fn get_score(state: &State_) -> i32 {
    let get_piece_advantage_score = |piece: u8| {
        [1, 4, 5, 100, 10]
            .into_iter()
            .enumerate()
            .map(|(i, advantage)| if piece & 1 << i != 0 { advantage } else { 0 })
            .sum::<i32>()
    };

    let ally_piece_advantage_score = state.pieces
        .iter()
        .enumerate()
        .map(|(i, piece)| if state.ownership & 1 << i != 0 { get_piece_advantage_score(*piece) } else { 0 })
        .sum::<i32>();

    let enemy_piece_advantage_score = state.pieces
        .iter()
        .enumerate()
        .map(|(i, piece)| if state.ownership & 1 << i == 0 { get_piece_advantage_score(*piece) } else { 0 })
        .sum::<i32>();

    ally_piece_advantage_score - enemy_piece_advantage_score
}

const MAX_DEPTH: i32 = 8;

fn alpha_beta(state: &State_, depth: i32, alpha: i32, beta: i32) -> (i32, Option<(u8, u8)>) {
    if Game_::won(state) {
        return ( 1_000 + depth, None)
    }

    if Game_::lost(state) {
        return (-1_000 - depth, None)
    }

    if depth == 0 {
        return (get_score(state), None)
    }

    let mut alpha = alpha;
    let mut action = None;

    for action_prime in Game_::legal_actions(state) {
        let state_prime = Game_::next_state(state, action_prime);
        let alpha_prime = -alpha_beta(&state_prime, depth - 1, -beta, -alpha).0;

        if alpha_prime > alpha {
            alpha = alpha_prime;
            action = Some(action_prime);
        }

        if alpha >= beta {
            break;
        }
    }

    (alpha, action)
}

#[wasm_bindgen(js_name = getAction)]
pub fn get_action(state: &State) -> Action {
    alpha_beta(&state.state, MAX_DEPTH, -9_999, 9_999).1.unwrap().into()
}
