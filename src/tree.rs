

use std::{
    hash::{Hash, Hasher},
    fmt::{Display, Formatter},
    ops::{Index, IndexMut},
    collections::HashMap,
};

pub const BOARD_SIZE: (usize, usize) = (7, 6);


// ===================== PLAYER ENUM =====================
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Player {
    Red,
    Yellow,
    Empty,
}

impl Player {
    fn flip(&self) -> Self {
        match self {
            Player::Red => Player::Yellow,
            Player::Yellow => Player::Red,
            _ => panic!("Turn is invalid. Turn: {:?}", self),
        }
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Player::Red    => write!(f, "🔴")?,
            Player::Yellow => write!(f, "🟡")?,
            Player::Empty  => write!(f, "  ")?,
        };
        Ok(())
    }
}


// ===================== GRID TYPE =====================
type Grid = Vec<Vec<Player>>;


// ===================== BOARD WRAPPER =====================
#[derive(Clone, Debug, Eq, PartialOrd, Ord)]
pub struct Board(pub Grid);

impl Index<usize> for Board {
    type Output = Vec<Player>;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

impl IndexMut<usize> for Board {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.0[idx]
    }
}

impl Board {
    pub fn empty() -> Self {
        let mut grid = Vec::new();
        for _x in 0..BOARD_SIZE.0 {
            grid.push(vec![Player::Empty; BOARD_SIZE.1]);
        }
        Board(grid)
    }

    pub fn play(&mut self, col: usize, player: Player) -> Option<usize> {
        for y in 0..BOARD_SIZE.1 {
            if self[col][y] == Player::Empty {
                self[col][y] = player;
                return Some(y);
            }
        }
        None
    }

    // Returns the canonical (lexicographically smallest) of the board and its mirror
    pub fn canonical(&self) -> Self {
        let original = &self.0;
        let mirror = original.iter().rev().cloned().collect::<Grid>();
        if original < &mirror { Board(original.clone()) } else { Board(mirror) }
    }

    pub fn from_turn(&self, col: usize, player: Player) -> Option<Self> {
        let mut new_board = self.clone();
        if let Some(_) = new_board.play(col, player) {
            Some(new_board)
        } else {
            None
        }
    }
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.canonical().0 == other.canonical().0
    }
}

impl Hash for Board {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for col in self.canonical().0.iter() {
            for cell in col {
                cell.hash(state);
            }
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let board = self.canonical();
        for y in (0..BOARD_SIZE.1).rev() {
            write!(f, "|")?;
            for x in 0..BOARD_SIZE.0 {
                write!(f, "{}", board[x][y])?;
            }
            writeln!(f)?;
        }
        for _ in 0..BOARD_SIZE.0 {
            write!(f, "  ︥")?;
        }
        Ok(())
    }
}


// ===================== RESULT ENUM =====================
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Result {
    Win(Player),
    Draw,
    Ongoing,
}

impl Result {
    fn from_board(board: &Board) -> Self {
        // Check horizontal, vertical, and both diagonals for 4 in a row
        for x in 0..BOARD_SIZE.0 {
            for y in 0..BOARD_SIZE.1 {
                let player = board[x][y];
                if player == Player::Empty {
                    continue;
                }
                // Horizontal
                if x + 3 < BOARD_SIZE.0
                    && board[x + 1][y] == player
                    && board[x + 2][y] == player
                    && board[x + 3][y] == player
                {
                    return Result::Win(player);
                }
                // Vertical
                if y + 3 < BOARD_SIZE.1
                    && board[x][y + 1] == player
                    && board[x][y + 2] == player
                    && board[x][y + 3] == player
                {
                    return Result::Win(player);
                }
                // Diagonal /
                if x + 3 < BOARD_SIZE.0 && y + 3 < BOARD_SIZE.1
                    && board[x + 1][y + 1] == player
                    && board[x + 2][y + 2] == player
                    && board[x + 3][y + 3] == player
                {
                    return Result::Win(player);
                }
                // Diagonal \
                if x >= 3
                    && y + 3 < BOARD_SIZE.1
                    && board[x - 1][y + 1] == player
                    && board[x - 2][y + 2] == player
                    && board[x - 3][y + 3] == player
                {
                    return Result::Win(player);
                }
            }
        }
        // Check for draw (no empty spaces)
        if board.0.iter().all(|col| col.iter().all(|&cell| cell != Player::Empty)) {
            Result::Draw
        } else {
            Result::Ongoing
        }
    }
}


// ===================== STATEINDEX WRAPPER =====================
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct StateIndex(pub usize);

impl Display for StateIndex {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub board: Board,
    turn: Player,
    pub result: Result,
    pub children: Vec<Option<StateIndex>>,
    pub index: Option<StateIndex>,
}

impl GameState {
    pub fn from_board(board: Board, turn: Player) -> Self {
        let result = Result::from_board(&board);

        GameState {
            board,
            turn,
            result,
            children: Vec::new(),
            index: None,
        }
    }

    fn from_turn(&self, col: usize) -> Option<Self> {
        let board = self.board.from_turn(col, self.turn);
        // If that move is valid ...
        if let Some(new_board) = board {
            let new_result = Result::from_board(&new_board);
            let new_turn = self.turn.flip();

            let state = GameState {
                board: new_board,
                turn: new_turn,
                result: new_result,
                children: Vec::new(),
                index: None,
            };
            Some(state)
        } else {
            None
        }
    }

    pub fn ok_children(&self) -> impl Iterator<Item = &StateIndex> {
        self.children.iter().filter_map(|c| c.as_ref())
    }

    /// Counts the number of pieces for each player on the board: (red, yellow)
    pub fn count_pieces(&self) -> (usize, usize) {
        let mut red_count = 0;
        let mut yellow_count = 0;
        for col in &self.board.0 {
            for &cell in col {
                match cell {
                    Player::Red    => red_count += 1,
                    Player::Yellow => yellow_count += 1,
                    _              => {},
                }
            }
        }
        (red_count, yellow_count)
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match &self.index {
            None                   => write!(f, "State: ?")?,
            Some(idx) => write!(f, "State: {}", idx)?,
        }
        writeln!(f, "{}", self.board)?;
        write!(f, "Goes to {}, ", self.children.len())?;
        match self.result {
            Result::Ongoing        => write!(f, "its {}'s turn", self.turn),
            Result::Win(p) => write!(f, "{} won!", p),
            Result::Draw           => write!(f, "impressive they drew waow"),
        }
    }
}



// ===================== TREE STRUCT =====================
pub struct Tree {
    pub root_index: StateIndex,
    pub nodes: Vec<GameState>,
    map: HashMap<Board, StateIndex>,
}

impl Tree {
    pub fn from_root(root: &GameState) -> Self {
        let root_index = StateIndex(0);

        let mut root = root.clone();
        root.index = Some(root_index.clone());

        let map = HashMap::new();

        Tree {
            root_index,
            nodes: vec![root],
            map,
        }
    }

    pub fn num_children(&self, index: &StateIndex) -> usize {
        self.nodes[index.0].children.len()
    }

    pub fn iter_children(&self, index: &StateIndex) -> impl Iterator<Item = &Option<StateIndex>> {
        self[index].children.iter()
    }

    pub fn iter_ok_children(&self, index: &StateIndex) -> impl Iterator<Item = &StateIndex> {
            self[index]
                .children
                .iter()
                .filter_map(|c| c.as_ref())
        }

    pub fn get_board(&self, index: &StateIndex) -> &Board {
        &self[index].board
    }

    // fn dive(&self) -> Option<(&Tree, &GameState)> {
    //     if self.nodes.is_empty() {
    //         return None;
    //     }
    //     let root_node = &self.nodes[self.root.0];
    //     Some((self, root_node))
    // }

    pub fn count_children(&self) -> usize {
        self.count_sub_children(&StateIndex(0))
    }

    fn count_sub_children(&self, si: &StateIndex) -> usize {
        self.iter_children(si).map(|child| {
            if let Some(child) = child {
                1 + self.count_sub_children(child)
            } else {
                0
            }
        }).sum::<usize>()
    }

    fn next_index(&self) -> StateIndex {
        StateIndex(self.nodes.len())
    }

    pub fn explore(&mut self, depth: u32) {
        self.find_children(self.root_index.clone(), depth);
    }

    pub fn explore_further(&mut self, depth: u32, head: &StateIndex) {
        let n = self.nodes.len();
        self.find_children(*head, depth);
        println!("Added {} nodes", self.nodes.len() - n);
    }

    fn find_children(
        &mut self,
        state_index: StateIndex,
        depth: u32,
    ) {
        if depth == 0 || self[&state_index].result != Result::Ongoing {
            return;
        }

        for x in 0..BOARD_SIZE.0 {
            if let Some(mut new_state) = self[&state_index].from_turn(x) {
                if let Some(&idx) = self.map.get(&new_state.board) {
                    new_state.index = Some(idx);
                    self[&state_index].children.push(Some(idx));
                    // return;
                } else {
                    let new_index = self.next_index();
                    new_state.index = Some(new_index.clone());
                    self[&state_index].children.push(Some(new_index.clone()));
                    self.map.insert(new_state.board.canonical(), new_index);
                    self.nodes.push(new_state);
                    self.find_children(new_index, depth - 1);
                }
            } else { // Invalid move, but still need to add a placeholder
                self[&state_index].children.push(None);
            }
        }
    }
}

impl Index<usize> for Tree {
    type Output = GameState;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.nodes[idx]
    }
}

impl Index<StateIndex> for Tree {
    type Output = GameState;
    fn index(&self, idx: StateIndex) -> &Self::Output {
        &self.nodes[idx.0]
    }
}
impl Index<&StateIndex> for Tree {
    type Output = GameState;
    fn index(&self, idx: &StateIndex) -> &Self::Output {
        &self.nodes[idx.0]
    }
}

impl IndexMut<StateIndex> for Tree {
    fn index_mut(&mut self, idx: StateIndex) -> &mut Self::Output {
        &mut self.nodes[idx.0]
    }
}
impl IndexMut<&StateIndex> for Tree {
    fn index_mut(&mut self, idx: &StateIndex) -> &mut Self::Output {
        &mut self.nodes[idx.0]
    }
}











// #[allow(dead_code)]
// fn play_game(game: &mut GameState) {
//     while game.result == Result::Ongoing {
//         println!("{}", game);
//         let mut input = String::new();
//         println!("Enter column to play (0-{}): ", BOARD_SIZE.0 - 1);
//         std::io::stdin().read_line(&mut input).expect("Failed to read line");
//         let col: usize = input.trim().parse().expect("Please enter a number");
        
//         game.play_col(col);
//         game.flip_turn();
//     }
// }

// #[allow(dead_code)]
// fn display_full_tree(start_node: &GameNode) {
//     for child in start_node.children.iter() {
//         println!("{}", child);
//         if !child.children.is_empty() {
//             println!("goes to:\n");
//             display_full_tree(child);
//         }
//     }
// }


/*
fn main() {
    let eb = empty_board();
    let mut game = GameNode::from_board(eb, Player::Red);

    let mut table: HashMap<BoardKey, Rc<GameNode>> = HashMap::new();
    table.insert(BoardKey(game.board.clone()), Rc::new(game.clone()));
    
    // Update the game passed in
    let now = std::time::Instant::now();
    find_children(&mut game, 9, &mut table);
    println!("Tree gen took {:?}", now.elapsed());

    println!("Total unique nodes: {}", table.len());
    println!("Total children: {}", game.count_children());

    let file = std::fs::File::create("tree.json");


    // display_full_tree(&game);


    // let all_winning_nodes = table.values().filter(|node| matches!(node.result, Result::Draw)).collect::<Vec<&Rc<GameNode>>>();
    // for n in all_winning_nodes.iter() {
    //     println!("{}", n);
    // }

    // for n in table.values() {
    //     println!("{}", n);
    // }

    // Random table entry
    // println!("Random table entry:\n{}", table.values().next().unwrap());

}
*/