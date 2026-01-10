use pyo3::pymodule;

// Pythonと連携するためのモジュールです。PettingZooの環境作成の補助をします。PettingZooの環境そのものは、python/quantum_animal_shogi/__init__.pyを参照してください。

#[pymodule]
mod quantum_animal_shogi {
    use std::iter::empty;

    use itertools::Itertools;
    use nalgebra::SMatrix;
    use ndarray::{Array1, Array2};
    use numpy::{IntoPyArray, PyReadonlyArray2};
    use pyo3::{Bound, PyAny, PyResult, Python, pyclass, pymethods, types::{PyAnyMethods, PyDict}};

    use crate::{Game, State, bits};

    // 観測します。RustのStateのままでも良いのですけど、Pythonで観測しやすい（と思われる）形に変換しておきます。

    fn observation<'py>(state: &State, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        // 観測結果そのものは"observation"に入れ、合法手を観測結果の"action_mask"に入れるのがPettingZooのおすすめみたいなので、Dictを作成します。

        let result = PyDict::new(py);  // PyDictはmutでなくても更新できちゃいます。。。

        result.set_item(
            "observation",
            {
                // "observation"はNumPy配列（Dictではない）がPettingZooのおすすめみたいなので、NumPy配列を作成します。

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
                // "action_mask"は1次元のMultiBinaryがPettingZooのおすすめみたいなので、選択可能なアクションのインデックスをTrueにしたNumPy配列を作成します。1次元のMultiBinaryにするために、アクションは(u8, u8)ではなく、u16にします。で、action.0 << 4 | action.1だと膨大な数の配列になってしまうので、action.0 * 12 + action.1にします。

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

        // ターン。

        result.set_item(
            "turn",
            state.turn
        )?;

        Ok(result)
    }

    // PettingZooのAECEnvを委譲で作成可能にするためのクラスです。

    #[pyclass]
    #[derive(Clone, Copy)]
    struct RawEnvironment {
        state: State
    }

    #[pymethods]
    impl RawEnvironment {
        // コンストラクタです。

        #[new]
        fn new() -> Self {
            Self {
                state: Game::initial_state()
            }
        }

        // 観測結果からRawEnvironmentを作成します。

        #[staticmethod]
        fn from_observation(observation: PyReadonlyArray2<f32>, turn: u16) -> Self {
            let observation: SMatrix<f32, 9, 20>  = SMatrix::from_column_slice(observation.as_slice().unwrap());

            let indices = empty()
                .chain(observation.column_iter().enumerate().filter_map(|(index, column)| if column[5] == 1.0 { Some(index) } else { None }))
                .chain(observation.column_iter().enumerate().filter_map(|(index, column)| if column[6] == 1.0 { Some(index) } else { None }))
                .collect_array::<8>()
                .unwrap();

            let pieces = indices.iter()
                .map(|index| observation.column(*index))
                .map(|column| (column[0] as u8) << 0 | (column[1] as u8) << 1 | (column[2] as u8) << 2 | (column[3] as u8) << 3 | (column[4] as u8) << 4)
                .collect_array::<8>()
                .unwrap();

            let ownership = indices.iter()
                .enumerate()
                .filter_map(|(i, index)| if observation.column(*index)[7] == 1.0 { Some(i) } else { None })
                .fold(0, |acc, i| acc | 1 << i);

            let bit_boards = indices.iter()
                .map(|index| if *index < 12 { 1 << (12 - 1 - *index) } else { 0 })
                .collect_array::<8>()
                .unwrap();

            Self {
                state: State { pieces, ownership, bit_boards, turn }
            }
        }

        // 1ステップ進め、報酬を返します。

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
                return -1.0;  // 不正なアクションは反則負けとします。
            };

            self.state = next_state;

            if Game::lost(&self.state) {
                1.0  // 次の状態での手番は敵なので、敵が負けたら自分の勝ちになります。
            } else {
                0.0
            }
        }

        // 環境をリセットします。

        fn reset(&mut self) {
            self.state = Game::initial_state();
        }

        // 観測を実施します。

        fn observe<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
            observation(&self.state, py)
        }

        // 勝敗が決した後に盤面を観測できるよう、回転させた状態での観測を実施します。

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

        // 負けたかどうかを取得します。

        fn lost(&self) -> bool {
            Game::lost(&self.state)
        }

        // Pythonで状態として保存でき量にするために、copyとdeepcopy、piekleに対応させます。

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

        // デバッグ用に、盤面を文字列化します。

        fn __str__(&self) -> String {
            self.state.to_string()
        }
    }
}
