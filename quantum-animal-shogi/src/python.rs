use pyo3::pymodule;

#[pymodule]
mod quantum_animal_shogi {
    use ndarray::{Array1, Array2};
    use numpy::{IntoPyArray};
    use pyo3::{Bound, PyAny, PyResult, Python, pyclass, pymethods, types::{PyAnyMethods, PyDict}};

    use crate::{Game, State, bits};

    #[pyclass]
    #[derive(Clone, Copy)]
    struct _Environment {
        state: State
    }

    #[pymethods]
    impl _Environment {
        #[new]
        fn new() -> Self {
            _Environment {
                state: Game::initial_state()
            }
        }

        fn step(&mut self, action: i32) -> f32 {
            let action = ((action / (4 * 3)) as u8, (action % (4 * 3)) as u8);
            let (next_state, reward) = Game::next_state(&self.state, action);

            self.state = next_state;

            reward
        }

        fn reset(&mut self) {
            self.state = Game::initial_state();
        }

        fn observe<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
            let result = PyDict::new(py);

            result.set_item(
                "observation",
                {
                    // チャンネルは、（駒種（ひよこ、きりん、ぞう、ライオン、にわとり）＋由来（先手、後手））×所有者（自分、敵）の14チャンネルです。

                    let mut result = Array2::<f32>::zeros((4 * 3 + 8, (5 + 2) * 2));

                    // 盤面。

                    {
                        for index in 0..8_usize {
                            let bit_board = self.state.bit_boards[index];

                            if bit_board == 0 {
                                continue;
                            }

                            let i = (4 * 3 - 1 - bit_board.trailing_zeros()) as usize;

                            let channel_index = if self.state.ownership & 1 << index != 0 { 0 } else { 7 };

                            for piece_bit in bits(self.state.pieces[index]) {
                                result[[i, channel_index + piece_bit]] = 1.0;
                            }

                            result[[i, channel_index + 5 + if (0..4).contains(&index) { 0 } else { 1 }]] = 1.0;
                        }
                    }

                    // 持ち駒。

                    {
                        let mut i_0 = 4 * 3;
                        let mut i_1 = 4 * 3 + 8 - 1;

                        for index in 0..8_usize {
                            let bit_board = self.state.bit_boards[index];

                            if bit_board != 0 {
                                continue;
                            }

                            let owned = self.state.ownership & 1 << index != 0;

                            let i = if owned { i_0 } else { i_1 };
                            let channel_index = if owned { 0 } else { 7 };

                            for piece_bit in bits(self.state.pieces[index]) {
                                result[[i, channel_index + piece_bit]] = 1.0;
                            }

                            result[[i, channel_index + 5 + if (0..4).contains(&index) { 0 } else { 1 }]] = 1.0;

                            if owned {
                                i_0 += 1;
                            } else {
                                i_1 -= 1;
                            }
                        }
                    }

                    result
                }.into_pyarray(py)
            )?;

            // 合法手。

            result.set_item(
                "action_mask",
                {
                    let mut result = Array1::<i8>::zeros(((4 * 3) + 8) * (4 * 3));

                    for action in Game::legal_actions(&self.state) {
                        result[(action.0 as usize) * (4 * 3) + action.1 as usize] = 1
                    }

                    result
                }.into_pyarray(py)
            )?;

            Ok(result)
        }

        #[getter]
        fn turn(&self) -> u16 {
            self.state.turn
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
