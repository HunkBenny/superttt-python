use core::panic;
use numpy::Ix1;
use numpy::PyArray;
use numpy::PyArrayMethods;
use numpy::ToPyArray;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyAnyMethods;
use pyo3::types::PyInt;
use pyo3::types::PyTuple;
use std::fmt::format;
use std::string;
use std::sync::Arc;
use superttt::game::play_game_multiple_threads as play_game_multiple_threads_rs;
use superttt::game::player::Player;
use superttt::game::player::RandomBot;
use superttt::game::state::HasState;
use superttt::game::state::State;

/// Functions that should be accessible in Python:
///
/// GAMES:
/// Creating players (randombot ok for now)
/// Creating games
///
use superttt::game::Game;
use superttt::game::MoveError;

#[pyclass(name="State",freelist=500)]
/// Represent a Pythonlike state. The current board, next player, and all possible moves.
/// Uses Numpy arrays when possible.
/// 
/// Important note:
/// * board - this attribute is a 1D-array. The first 9 elements represent the first field (upper left), the next 9 the second field (upper center) and so on
/// ```text
/// Layout:
///  0  1  2        9  10  11       18  19  20 
///  3  4  5       12  13  14       21  22  23 
///  6  7  8       15  16  17       24  25  26
///  
/// 27  28  29     36  37  38       45  46  47 
/// 30  31  32     39  40  41       48  49  50 
/// 33  34  35     42  43  44       51  52  53 
/// 
/// 54  55  56     63  64  65       72  73  74 
/// 57  58  59     66  67  68       75  76  77 
/// 60  61  62     69  70  71       78  79  80
/// ```
pub struct PyState {
    #[pyo3(get, set)]
    /// TEST
    board: Py<PyArray<i8, Ix1>>,
    #[pyo3(get, set)]
    player_id: i8,
    #[pyo3(get, set)]
    possible_moves: Py<PyArray<i8, Ix1>>,
}

#[pymethods]
impl PyState {
    // macros?
    const vb: char = '\u{002502}'; // vertical bar
    const hb: char = '\u{002500}'; // horizontal bar
    const c: char = '\u{00253C}'; // cross (corner of roster)
    const fvb: char = '\u{002503}'; // fat vertical bar
    const fhb: char = '\u{002501}'; // fat horizontal bar
    const fc: char = '\u{00254B}'; // fat cross
    // this might seem like a bad practice; however constants are inlined, so I believe that the performance hit will be negligble
    // The benefit is more readable code. 0-9 belong to the same subcell, not to the same line. This is why I opt to use this constant array.
    const idx: [usize; 81] = [
        0, 1, 2, 9, 10, 11, 18, 19, 20,
        3, 4, 5, 12 ,13, 14, 21, 22, 23,
        6, 7, 8, 15, 16, 17, 24, 25, 26,
        27, 28, 29, 36, 37, 38, 45, 46, 47,
        30, 31, 32, 39, 40, 41 ,48, 49, 50,
        33, 34, 35, 42, 43, 44, 51, 52 ,53,
        54, 55, 56, 63, 64, 65, 72, 73, 74,
        57, 58, 59, 66, 67, 68, 75, 76, 77,
        60, 61, 62, 69, 70, 71, 78, 79, 80
        ];

    fn __str__<'py>(&self, py: Python<'py>) -> String {
        let mut out = "Board:\n".to_owned();
        unsafe {
            //* print board:
            // first line: do not add newline
            for i in 0..9 {
                // do not print in the front of the line
                if i != 0 {
                    if i % 3 != 0 {
                        // print small Self::vb
                        out += &format!("{}", Self::vb);
                    } else {
                        // divider between subfields:
                        out += &format!("{}", Self::fvb);
                    }
                }
                out += &format!(" {} ", &self.board.bind(py).get(*Self::idx.get(i).unwrap()).unwrap().to_string());
            }
            // add newlines
            let mut num_newlines = 1;
            for i in 9..81 {
                if i % 9 == 0 {
                    // newline
                    out += "\n";
                    //? there are 9 cells in a line, but we add 8 bars between these fields: so 17 characters
                    // * pattern: Self::hb cross Self::hb cross Self::hb
                    if num_newlines % 3 == 0 {
                        out += &format!("{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}", Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fc, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fc, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb, Self::fhb);
                    } else {
                        out += &format!("{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}", Self::hb, Self::hb, Self::hb, Self::c, Self::hb, Self::hb, Self::hb, Self::c, Self::hb, Self::hb, Self::hb, Self::fvb, Self::hb, Self::hb, Self::hb, Self::c, Self::hb, Self::hb, Self::hb, Self::c, Self::hb, Self::hb, Self::hb, Self::fvb, Self::hb, Self::hb, Self::hb, Self::c, Self::hb, Self::hb, Self::hb, Self::c, Self::hb, Self::hb);
                    }
                    out += "\n";
                    num_newlines += 1;
                } else if i % 3 != 0 {
                    // small vertical bar
                    out += &format!("{}", Self::vb);
                } else {
                    // fat vertical bar (between subfields)
                    out += &format!("{}", Self::fvb);
                }
                out += &format!(" {} ", &self.board.bind(py).get(*Self::idx.get(i).unwrap()).unwrap().to_string());
            }
            //* print next player:
            out += &format!("\n\nNext Player: {}\n", self.player_id);
            //* print possible moves:
            out += &format!("\nPossible Moves: {}\n", self.possible_moves);
        };
        out
    }
}

#[pyclass(name="Checkpoint")]
pub struct PyCheckpoint(State);

#[pymethods]
/// This should only be created in other methods (after making a move)
/// Still need to think about where this should be implemented.
///
/// Note: We could give the decision where to use it to the python-programmers?
/// Perhaps use serde?
impl PyCheckpoint {
    fn get_odds(&self) -> [f64; 3] {
        self.0.get_odds()
    }
    fn __str__(&self) -> String {
        format!("{}", self.0)
    }
}

struct PyMoveError(MoveError);

impl From<MoveError> for PyMoveError {
    fn from(value: MoveError) -> Self {
        PyMoveError(value)
    }
}

impl std::convert::From<PyMoveError> for PyErr {
    fn from(error: PyMoveError) -> Self {
        PyValueError::new_err(error.0.to_string())
    }
}

#[pyclass(name="Game")]
/// Game class
///
/// # Methods:
/// * get_available_moves() - Returns a vector containing the cells that are `valid` in the current state.
/// * make_move(input, player_id_authentication)
pub struct PyGame(Game);

#[pymethods]
impl PyGame {
    #[new]
    fn __new__(id: u32) -> PyGame {
        PyGame(Game::new(id, false))
    }

    fn get_available_moves(&self) -> Vec<i8> {
        self.0.get_available_moves()
    }

    /// Make a move. This does nothing else.
    ///
    ///
    /// ## Args
    /// * `input` - the cell that the current player makes a move in.
    /// * `player_id_authentication` - the id of the player that is making the move. This should be one of: {1, 2}
    ///
    fn make_move(&mut self, input: i8, player_id_authentication: i8) -> Result<bool, PyMoveError> {
        let not_finished = self.0.make_move(input, player_id_authentication)?;
        Ok(not_finished)
    }

    /// Returns the id of the winner of the game.
    ///
    /// If it is equal to 0, then the game ended in a draw.
    ///
    fn get_winner(&self) -> i8 {
        match self.0.get_winner() {
            Some(s) => s,
            None => 0,
        }
    }

    /// Create a checkpoint. This can be saved somewhere and loaded in at a later point in time.
    fn make_checkpoint(&self) -> PyCheckpoint {
        PyCheckpoint(self.0.make_state())
    }

    /// Load in a checkpoint from a PyCheckpoint object.
    ///
    /// ## Args
    /// * `checkpoint` - checkpoint to load in.
    fn load_checkpoint(&mut self, checkpoint: &PyCheckpoint) -> () {
        self.0.load_state(&checkpoint.0);
    }

    /// Create a `PyState`-object.
    /// This object is a simple representation of the current game.
    fn make_state<'py>(&self, py: Python<'py>) -> PyState {
        PyState {
            board: self.0.get_board().to_pyarray(py).unbind(),
            player_id: self.0.get_previous_move()[0],
            possible_moves: self.0.get_available_moves().to_pyarray(py).unbind(),
        }
    }

    /// Python __str__ method
    fn __str__(&self) -> String {
        format!("{}", self.0)
    }
}

#[pyclass(subclass, name="Player")]
pub struct PyPlayer(Arc<dyn Player + Send + Sync>);

#[pymethods]
impl PyPlayer {
    #[new]
    fn __new__(id: i8) -> PyPlayer {
        PyPlayer(Arc::new(RandomBot::new(id)))
    }

    /// Make a move with the current player.
    /// The default implementation is naive (makes a random move).
    ///
    /// Override this method to have your custom heuristic.
    fn make_move<'py>(&self, game: &Bound<'py, PyGame>) -> () {
        // let args = PyTuple::new(py, &[game]).unwrap();
        // slf.call_method1(py, "make_move", args).unwrap();

        self.0.make_move(&mut game.borrow_mut().0);
    }

    /// Increment the number of wins for a player by one.
    fn increment_wins(&self) -> () {
        self.0.increment_wins();
    }
    /// Get the total number of wins for a player.
    fn get_wins(&self) -> u32 {
        self.0.get_wins()
    }
}

#[pyfunction]
/// Play multiple games in a single-threaded environment
fn play_multiple_games<'py>(
    py: Python<'py>,
    player1: Bound<'py, PyPlayer>,
    player2: Bound<'py, PyPlayer>,
    number_of_games: Bound<'py, PyInt>,
) {
    let num: u32 = match number_of_games.extract() {
        Ok(o) => o,
        Err(e) => {
            panic!("{}\nCannot simulate more games than what a u32 allows. You definitely do not need to run more games than that.",e)
        }
    };
    for _ in 0..num {
        let game = Bound::new(py, PyGame::__new__(0)).unwrap();
        play_game(py, &player1, &player2, game);
    }
}

#[pyfunction]
/// Play one game between two player objects.
/// Delegates to overridden make_move method if it exists.
///
/// Todo: add caching for speeding up perhaps?
fn play_game<'py>(
    py: Python<'py>,
    player1: &Bound<'py, PyPlayer>,
    player2: &Bound<'py, PyPlayer>,
    game: Bound<'py, PyGame>,
) {
    let mut player_turn = game.try_borrow_mut().unwrap().0.get_previous_move()[0];
    while !game.try_borrow_mut().unwrap().0.is_finished() {
        if player_turn % 2 == 1 {
            // if player contains make move method:
            // check this with the getattr method
            match (
                player1.is_exact_instance_of::<PyPlayer>(),
                player1.is_instance_of::<PyPlayer>(),
            ) {
                (false, true) => {
                    // this is what we want! This means that the object is a subclass of pyplayer. i.e. it is implemented by the python user
                    let args = PyTuple::new(py, &[&game]).unwrap();
                    player1.call_method1("make_move", args).unwrap();
                }
                _ => {
                    player1.try_borrow_mut().unwrap().make_move(&game);
                }
            }
            player_turn = 2;
        } else {
            match (
                player2.is_exact_instance_of::<PyPlayer>(),
                player2.is_instance_of::<PyPlayer>(),
            ) {
                (false, true) => {
                    // this is what we want! This means that the object is a subclass of pyplayer. i.e. it is implemented by the python user
                    let args = PyTuple::new(py, &[&game]).unwrap();
                    player2.call_method1("make_move", args).unwrap();
                }
                _ => {
                    player2.try_borrow_mut().unwrap().make_move(&game);
                }
            }
            player_turn = 1;
        }
    }
    match game.try_borrow_mut().unwrap().0.get_winner() {
        Some(s) => {
            if s == 1 {
                player1.try_borrow_mut().unwrap().0.increment_wins();
            } else if s == 2 {
                player2.try_borrow_mut().unwrap().0.increment_wins();
            }
        }
        None => {}
    }
}

#[pyfunction]
fn play_game_multiple_threads(
    player1: &PyPlayer,
    player2: &PyPlayer,
    number_of_games: u32,
    number_of_threads: u32,
) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .build()
        .expect("Error: Creation of Multi-Threaded environment failed.");
    let handle = rt.spawn(play_game_multiple_threads_rs(
        player1.0.clone(),
        player2.0.clone(),
        number_of_games,
        number_of_threads,
    ));
    rt.block_on(handle).unwrap();
}
// #[pyfunction]
// async fn play_game_multiple_threads<'a>(
//     py: Python<'a>,
//     player1: &'_ PyPlayer,
//     player2: &'_ PyPlayer,
//     number_of_games: u32,
//     number_of_threads: u32,
// ) -> PyResult<&'a PyAny> {
//     pyo3_asyncio::tokio::future_into_py(py, async {
//         play_game_multiple_threads_rs(
//             player1.0.clone(),
//             player2.0.clone(),
//             number_of_games,
//             number_of_threads,
//         ).await;
//         Ok(Python::with_gil(|py| py.None()))
//     })
// }

/// A Python module implemented in Rust.
#[pymodule]
fn superttt_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGame>()?;
    m.add_class::<PyPlayer>()?;
    m.add_class::<PyCheckpoint>()?;
    m.add_class::<PyState>()?;
    m.add_function(wrap_pyfunction!(play_game, m)?)?;
    m.add_function(wrap_pyfunction!(play_game_multiple_threads, m)?)?;
    m.add_function(wrap_pyfunction!(play_multiple_games, m)?)?;
    Ok(())
}
