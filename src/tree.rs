

use std::{
    rc::Rc,
    collections::HashMap,
    hash::{Hash, Hasher},
    fmt::{Display, Formatter},
};

const BOARD_SIZE: (usize, usize) = (4, 4);


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Player {
    Red,
    Yellow,
    Empty,
}

impl Player {
    fn flip(&self) -> Self {
        return match self {
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




type Board = Vec<Vec<Player>>;

#[derive(Clone, PartialEq, Eq)]
pub struct BoardKey(pub Board);

impl Hash for BoardKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for col in &self.0 {
            for cell in col {
                cell.hash(state);
            }
        }
    }
}

pub fn empty_board() -> Board {
    let mut board = Vec::new();
    for _x in 0..BOARD_SIZE.0 {
        board.push(vec![Player::Empty; BOARD_SIZE.1]);
    }
    board
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Result {
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
        if board.iter().all(|col| col.iter().all(|&cell| cell != Player::Empty)) {
            return Result::Draw
        } else {
            return Result::Ongoing
        }
    }
}



#[derive(Debug, Clone)]
pub struct GameNode {
    pub board: Vec<Vec<Player>>,
    turn: Player,
    result: Result,
    pub children: Vec<Rc<GameNode>>,
}

impl GameNode {
    pub fn from_board(board: Board, turn: Player) -> Self {
        let result = Result::from_board(&board);

        GameNode {
            board,
            turn,
            result,
            children: Vec::new(),
        }
    }

    fn play_col(&mut self, col: usize) {
        for y in 0..BOARD_SIZE.1 {
            if self.board[col][y] == Player::Empty {
                self.board[col][y] = self.turn;
                self.result = Result::from_board(&self.board);
                return
            }
        }
        panic!("Column {} is full", col);
    }

    fn board_from_turn(&self, col: usize) -> Option<Vec<Vec<Player>>> {
        let mut board = self.board.clone();

        for y in 0..BOARD_SIZE.1 {
            if board[col][y] == Player::Empty {
                board[col][y] = self.turn;
                return Some(board)
            }
        }
        None
    }

    fn flip_turn(&mut self) {
        self.turn = match self.turn {
            Player::Red => Player::Yellow,
            Player::Yellow => Player::Red,
            _ => panic!("This board's turn is Empty\n{}", self),
        }
    }

    pub fn count_children(&self) -> usize {
        self.children.iter().map(|child| child.count_children()).sum::<usize>() + self.children.len()
    }
}

impl Display for GameNode {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for y in (0..BOARD_SIZE.1).rev() {
            for x in 0..BOARD_SIZE.0 {
                write!(f, "{}", self.board[x][y])?;
            }
            write!(f, "\n")?;
        }
        write!(f, "Goes to {}, ", self.children.len())?;
        match self.result {
            Result::Ongoing        => write!(f, "its {}'s turn", self.turn),
            Result::Win(p) => write!(f, "{} won!", p),
            Result::Draw           => write!(f, "impressive they drew waow"),
        }
    }
}



#[allow(dead_code)]
fn play_game(game: &mut GameNode) {
    while game.result == Result::Ongoing {
        println!("{}", game);
        let mut input = String::new();
        println!("Enter column to play (0-{}): ", BOARD_SIZE.0 - 1);
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
        let col: usize = input.trim().parse().expect("Please enter a number");
        
        game.play_col(col);
        game.flip_turn();
    }
}





pub fn find_children(
    node: &mut GameNode,
    depth: u32,
    table: &mut HashMap<BoardKey, Rc<GameNode>>,
) -> Option<Rc<GameNode>> {
    let result = node.result;

    if depth == 0 || result != Result::Ongoing {
        return Some(Rc::new(node.clone()));
    }

    for x in 0..BOARD_SIZE.0 {
        // Try playing in column x
        let new_board_op = GameNode::board_from_turn(&node, x);
        
        // If play is valid ...
        if let Some(new_board) = new_board_op {
            // Get the key to this state in the table
            let key = BoardKey(new_board.clone());

            // If this state has already been searched ...
            let child_rc_op = if let Some(existing) = table.get(&key) {
                // Get the existing state from the table
                Some(Rc::clone(existing))
            } else {
                // Otherwise create the new game state node
                let mut new_node = GameNode::from_board(new_board, node.turn.flip());
                // Recurse into this node
                let searched_node_op = find_children(&mut new_node, depth-1, table);
                // If this recursion results in a new state ...
                if let Some(searched_node) = searched_node_op {
                    // Add the new state to the table
                    table.insert(key.clone(), searched_node);
                    Some(table.get(&key).unwrap().to_owned())
                } else {
                    // Ignore and stop the recursion from this node
                    None
                }
            };

            // If the child for column x is valid ...
            if let Some(child_rc) = child_rc_op {
                node.children.push(child_rc);
            }
        }
    }

    Some(Rc::new(node.clone()))
}





#[allow(dead_code)]
fn display_full_tree(start_node: &GameNode) {
    for child in start_node.children.iter() {
        println!("{}", child);
        if !child.children.is_empty() {
            println!("goes to:\n");
            display_full_tree(child);
        }
    }
}


/*


fn main() {
    let eb = empty_board();
    let mut game = GameNode::from_board(eb, Player::Red);

    let mut table: HashMap<BoardKey, Rc<GameNode>> = HashMap::new();
    table.insert(BoardKey(game.board.clone()), Rc::new(game.clone()));
    
    // Update the game passed in
    find_children(&mut game, 3, &mut table).unwrap();

    println!("Total unique nodes: {}", table.len());
    println!("Total children: {}", game.count_children());

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