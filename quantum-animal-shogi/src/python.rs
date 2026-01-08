use pyo3::pymodule;

#[pymodule]
mod quantum_animal_shogi {
    use ndarray::{Array1, Array2};
    use numpy::{IntoPyArray};
    use pyo3::{Bound, PyAny, PyResult, Python, pyclass, pymethods, types::{PyAnyMethods, PyDict}};

    use crate::{Game, State, bits};

    fn observation<'py>(state: &State, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let result = PyDict::new(py);

        result.set_item(
            "observation",
            {
                let mut result = Array2::<f32>::zeros((4 * 3 + 8, 5 + 2 + 2));  // [行(4)×列（3）＋持ち駒（自分と敵合わせて8）、駒種（ひよこ、きりん、ぞう、ライオン、にわとり）＋由来（先手、後手））＋所有者（自分、敵）]。

                // 盤面。

                for index in 0..8_usize {
                    let bit_board = state.bit_boards[index];

                    if bit_board == 0 {
                        continue;
                    }

                    let i = (4 * 3 - 1 - bit_board.trailing_zeros()) as usize;

                    for piece_bit in bits(state.pieces[index]) {
                        result[[i, piece_bit]] = 1.0;
                    }

                    result[[i, 5 + if (0..4).contains(&index) { 0 } else { 1 }]] = 1.0;
                    result[[i, 5 + 2 + if state.ownership & 1 << index != 0 { 0 } else { 1 }]] = 1.0;
                }

                // 自分の持ち駒。

                for (i, index) in bits(state.ownership).filter(|index| state.bit_boards[*index] == 0).enumerate().map(|(i, index)| (4 * 3 + i, index)) {
                    for piece_bit in bits(state.pieces[index]) {
                        result[[i, piece_bit]] = 1.0;
                    }

                    result[[i, 5 + if (0..4).contains(&index) { 0 } else { 1 }]] = 1.0;
                    result[[i, 5 + 2 + 0]] = 1.0;
                }

                // 敵の持ち駒。

                for (i, index) in bits(!state.ownership).filter(|index| state.bit_boards[*index] == 0).enumerate().map(|(i, index)| (4 * 3 + 8 - 1 - i, index)) {
                    for piece_bit in bits(state.pieces[index]) {
                        result[[i, piece_bit]] = 1.0;
                    }

                    result[[i, 5 + if (0..4).contains(&index) { 0 } else { 1 }]] = 1.0;
                    result[[i, 5 + 2 + 1]] = 1.0;
                }

                result.into_pyarray(py)
            }
        )?;

        // 合法手。

        result.set_item(
            "action_mask",
            {
                let mut result = Array1::<i8>::zeros(((4 * 3) + 8) * (4 * 3));

                for action in Game::legal_actions(&state) {
                    // Python側の座標系（Rust側では0は盤面の右下ですが、Python側では左上）に合うように、アクションを変更します。

                    let action = {
                        if action.0 < 4 * 3 {
                            (12 - 1 - action.0, 12 - 1 - action.1)
                        } else {
                            (action.0, 12 - 1 - action.1)
                        }
                    };

                    result[(action.0 as usize) * (4 * 3) + (action.1 as usize)] = 1
                }

                result.into_pyarray(py)
            }
        )?;

        Ok(result)
    }

    #[pyclass]
    #[derive(Clone, Copy)]
    struct RawEnvironment {
        state: State
    }

    #[pymethods]
    impl RawEnvironment {
        #[new]
        fn new() -> Self {
            RawEnvironment {
                state: Game::initial_state()
            }
        }

        fn step(&mut self, action: i32) -> f32 {
            // Python側の座標系（Rust側では0は盤面の右下ですが、Python側では左上）に合うように、アクションを変更します。

            let action = {
                let result = ((action / (4 * 3)) as u8, (action % (4 * 3)) as u8);

                if result.0 < 4 * 3 {
                    (12 - 1 - result.0, 12 - 1 - result.1)
                } else {
                    (result.0, 12 - 1 - result.1)
                }
            };

            let Some(next_state) = Game::next_state(&self.state, action) else {
                return -1.0;
            };

            self.state = next_state;

            if Game::won(&self.state) {
                1.0
            } else {
                0.0
            }
        }

        fn reset(&mut self) {
            self.state = Game::initial_state();
        }

        fn observe<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
            observation(&self.state, py)
        }

        fn observe_turned<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
            // 盤面を回転した状態を取得します。

            let state = {
                let mut result = self.state.clone();

                result.ownership = !result.ownership;
                result.bit_boards = result.bit_boards.map(|bit_board| bit_board.reverse_bits() >> 4);

                result
            };

            // 観測結果を取得し、リターンします。

            observation(&state, py)
        }

        fn __copy__(&self) -> Self {
            *self
        }

        fn __deepcopy__(&self) -> Self {
            *self
        }

        fn __getstate__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
            let result = PyDict::new(py);

            result.set_item("pieces", pyo3::types::PyBytes::new(py, &self.state.pieces))?;
            result.set_item("ownership", self.state.ownership)?;
            result.set_item("bit_boards", self.state.bit_boards.to_vec())?;
            result.set_item("turn", self.state.turn)?;

            Ok(result)
        }

        #[staticmethod]
        fn __setstate__(state: &Bound<'_, PyAny>) -> PyResult<Self> {
            let pieces = {
                let mut result = [0_u8; 8];
                result.copy_from_slice(&state.get_item("pieces")?.extract::<Vec<_>>()?);
                result
            };

            let ownership: u8 = state.get_item("ownership")?.extract()?;

            let bit_boards = {
                let mut result = [0_u16; 8];
                result.copy_from_slice(&state.get_item("bit_boards")?.extract::<Vec<_>>()?);
                result
            };

            let turn: u16 = state.get_item("turn")?.extract()?;

            Ok(
                Self {
                    state: State {
                        pieces,
                        ownership,
                        bit_boards,
                        turn
                    }
                }
            )
        }

        fn __str__(&self) -> String {
            self.state.to_string()
        }
    }
}
