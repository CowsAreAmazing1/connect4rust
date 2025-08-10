use std::fmt::{Display, Formatter};

mod tree_display;

const BOARD_SIZE: (usize, usize) = (7, 6);


#[derive(Debug, Copy, Clone, PartialEq)]
enum Player {
    Red,
    Yellow,
    Empty,
}

type Board = Vec<Vec<Player>>;

fn empty_board() -> Board {
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
struct GameNode {
    board: Vec<Vec<Player>>,
    turn: Player,
    result: Result,
    children: Vec<GameNode>,
}

impl GameNode {
    fn from_board(board: Board) -> Self {
        let result = Result::from_board(&board);

        GameNode {
            board,
            turn: Player::Red,
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

    fn from_turn(&mut self, col: usize) -> Self {
        let new_board = self.board.clone();
        let mut new_node = GameNode::from_board(new_board);
        new_node.play_col(col);
        new_node.flip_turn();

        self.children.push(new_node.clone());

        return new_node;
    }

    fn flip_turn(&mut self) {
        self.turn = match self.turn {
            Player::Red => Player::Yellow,
            Player::Yellow => Player::Red,
            _ => panic!("This board's turn is Empty\n{}", self),
        }
    }

    fn count_children(&self) -> usize {
        self.children.iter().map(|child| child.count_children()).sum::<usize>() + self.children.len()
    }
}

impl Display for GameNode {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for y in (0..BOARD_SIZE.1).rev() {
            for x in 0..BOARD_SIZE.0 {
                match self.board[x][y] {
                    Player::Red    => write!(f, "🔴")?,
                    Player::Yellow => write!(f, "🟡")?,
                    Player::Empty  => write!(f, "  ")?,
                };
            }
            write!(f, "\n")?;
        }
        write!(f, "Goes to {}", self.children.len())
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





fn find_children(game: &mut GameNode, depth: u32) {
    if depth == 0 || game.result != Result::Ongoing {
        return;
    }

    for x in 0..BOARD_SIZE.0 {
        let child = game.from_turn(x);
        game.children.push(child);
        find_children(&mut game.children.last_mut().unwrap(), depth - 1);
    }
}







fn main() {
    let eb = empty_board();
    let mut game = GameNode::from_board(eb);

    find_children(&mut game, 1);


    println!("Total children: {}", game.count_children());

    println!("Game tree:");
    for (i, c) in game.children.iter().enumerate() {
        println!("Child: {}: {}", i, c.children.len());
    }
}
