mod tree;
use tree::{Board, GameState, Player, Result, StateIndex, Tree};

use bisetmap::BisetMap;
use eframe::{App, NativeOptions, egui::Pos2, run_native};
use egui::{Color32, FontId, TextFormat};
use egui_graphs::SettingsNavigation;
use epaint::text::LayoutJob;
use petgraph::prelude::*;
use std::{collections::HashSet, time};
// use rand::prelude::*;

use crate::tree::BOARD_SIZE;

fn main() -> eframe::Result<()> {
    run_native(
        "grahpher",
        NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(BasicApp::new(cc)))),
    )
}

pub struct BasicApp {
    tree: Tree,
    graph: egui_graphs::Graph,
    connections: HashSet<(StateIndex, StateIndex)>,
    zoom_pan: bool,
    state_node_map: BisetMap<StateIndex, NodeIndex>,
    hovered_node: Option<NodeIndex>,
    target_index: Option<StateIndex>,
}

impl BasicApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let board = Board::empty();

        let game = GameState::from_board(board.canonical(), Player::Red);
        let mut tree = Tree::from_root(&game);

        let now = time::Instant::now();
        tree.explore(10);

        println!("Tree gen took {:?}", now.elapsed());
        println!("Total unique nodes: {}", tree.nodes.len());
        println!("Total children: {}", tree.count_children());

        println!("{}", &tree[&tree.root_index]);

        let dig = StableDiGraph::new();
        let graph = egui_graphs::Graph::from(&dig);

        let mut app = Self {
            tree,
            graph,
            connections: HashSet::new(),
            zoom_pan: false,
            state_node_map: BisetMap::new(),
            hovered_node: None,
            target_index: None,
        };

        app.start_game();
        app
    }

    fn start_game(&mut self) {
        let mut win_nodes = vec![false; self.tree.nodes.len()];

        let root_index = &self.tree.root_index;
        let target = &self.tree[root_index].board;

        prune_to_target(&self.tree, &mut win_nodes, root_index, target);

        add_to_graph(
            &self.tree,
            root_index,
            &mut self.graph,
            &mut self.state_node_map,
            &mut self.connections,
            &win_nodes,
            &mut vec![true; self.tree.nodes.len()],
            &mut rand::rng(),
        );

        self.target_index = Some(*root_index);
    }

    fn play_move(&mut self, col: usize) {
        if let Some(idx) = self.target_index {
            let state_idx_opt = self.tree[idx].children.get(col);
            if let Some(Some(next_idx)) = state_idx_opt {
                let root_index = self.tree.root_index;
                let new_state_index = self.tree[next_idx].index.unwrap();
                self.target_index = Some(new_state_index);

                let mut mask = vec![false; self.tree.nodes.len()];
                mask[0] = true; // always keep root

                let now = time::Instant::now();
                let target_board = self.tree[new_state_index].board.clone();
                println!(
                    "Pruning to target {}",
                    self.tree[&self.target_index.unwrap()]
                );
                prune_to_target(&self.tree, &mut mask, &root_index, &target_board);

                println!("Pruning took {:?}", now.elapsed());
                println!("Pruned to {} nodes", &mask.iter().filter(|&n| *n).count());

                // If masking, ensure the node being moved to is kept
                mask[new_state_index.0] = true;
                mask[idx.0] = true;
                let now = time::Instant::now();
                // self.tree.mask_nodes(&mask);
                println!("Masking took {:?}", now.elapsed());

                let now = time::Instant::now();
                add_to_graph(
                    &self.tree,
                    &root_index,
                    &mut self.graph,
                    &mut self.state_node_map,
                    &mut self.connections,
                    &mask,
                    &mut vec![true; self.tree.nodes.len()],
                    &mut rand::rng(),
                );
                println!("Graph generating took {:?}", now.elapsed());

                // self.tree.nodes.iter()
                //     .map(|n| {
                //         let (r, y) = n.count_pieces();
                //         r + y
                //     })
                //     .zip(mask)
                //     .for_each(|(pieces, keep)| print!("{}{} ", pieces, keep as u8));

                println!("{:?}", &self.target_index);
                println!("{}", &self.tree[idx].children.len());

                // let children_indices = self.tree[idx].children.clone();
                // println!("{:?}", &children_indices);
                // for other_children in children_indices {
                //     if let Some(other) = other_children {
                //         self.tree.explore_further(1, &other);
                //     }
                // }
            }
        }
    }
}

impl App for BasicApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // let mut widget = egui_graphs::DefaultGraphView::new(&mut self.g);
            // ui.add(&mut widget);

            type L =
                egui_graphs::LayoutForceDirected<egui_graphs::FruchtermanReingoldWithCenterGravity>;
            type S = egui_graphs::FruchtermanReingoldWithCenterGravityState;

            let mut widget = egui_graphs::GraphView::<_, _, _, _, _, _, S, L>::new(&mut self.graph)
                .with_navigations(
                    &SettingsNavigation::new()
                        .with_zoom_and_pan_enabled(self.zoom_pan)
                        .with_fit_to_screen_enabled(!self.zoom_pan),
                );
            ui.add(&mut widget);

            // // Force‑Directed (FR) #with Center Gravity
            // type L = egui_graphs::LayoutForceDirected<egui_graphs::FruchtermanReingoldWithCenterGravity>;
            // type S = egui_graphs::FruchtermanReingoldWithCenterGravityState;
            // let mut view = egui_graphs::GraphView::<_,_,_,_,_,_,S,L>::new(&mut self.g);
            // ui.add(&mut view);
        });
        egui::SidePanel::right("settings").show(ctx, |ui| {
            ui.label("Settings");
            ui.separator();
            ui.add(egui::Checkbox::new(&mut self.zoom_pan, "Zoom/Pan"));
            ui.separator();

            // ui.horizontal_centered(|ui| {
            //         ui.vertical_centered(|ui| {
            //             let texter = self.game.children[1].children[2].children[1].children[0].children[0].to_string();
            //             ui.label(egui::RichText::new(texter).size(20.0).color(Color32::RED));
            //         })
            //     }
            // )

            // Show hovered node info
            if let Some(idx) = self.graph.hovered_node() {
                if let Some(old) = self.hovered_node {
                    if old != idx {
                        self.hovered_node = Some(idx);
                    }
                } else {
                    self.hovered_node = Some(idx);
                }
            }

            if let Some(idx) = &self.hovered_node {
                if let Some(key) = self.state_node_map.rev_get(idx).first() {
                    ui.label(format!("Hovered Node Key: {}", key.0));
                    ui.label(board_to_layout_job(self.tree.get_board(key)));
                }
            } else {
                ui.label("Hovered Node Key: None");
            }

            ui.separator();

            // Show target node info
            if let Some(idx) = &self.target_index {
                ui.label(format!("Target Node Key: {}", idx.0));
                ui.label(board_to_layout_job(self.tree.get_board(idx)));
            }

            ui.horizontal(|ui| {
                for col in 0..BOARD_SIZE.0 {
                    if ui.button(format!("{}", col)).clicked() {
                        self.play_move(col);
                    }
                }
            })
        });
    }
}

fn board_to_layout_job(board: &Board) -> LayoutJob {
    let mut job = LayoutJob::default();
    let font_size = 20.0;

    for y in (0..BOARD_SIZE.1).rev() {
        for x in 0..BOARD_SIZE.0 {
            let color = match board[x][y] {
                Player::Red => Color32::RED,
                Player::Yellow => Color32::YELLOW,
                Player::Empty => Color32::BLACK,
            };
            job.append(
                "🔴", // appears in text color; larger than other circles
                0.0,
                TextFormat::simple(FontId::monospace(font_size), color),
            );
        }
        job.append(
            "\n",
            0.0,
            TextFormat::simple(FontId::monospace(font_size), Color32::BLACK),
        );
    }

    job
}

fn add_to_graph(
    tree: &Tree,
    state_index: &StateIndex,
    graph: &mut egui_graphs::Graph,
    state_node_map: &mut BisetMap<StateIndex, NodeIndex>,
    connections: &mut HashSet<(StateIndex, StateIndex)>,
    mask: &Vec<bool>,
    recheck: &mut Vec<bool>,
    rng: &mut impl rand::Rng,
) -> Option<NodeIndex> {
    if let Some(&b) = mask.get(state_index.0) {
        if !b {
            return None;
        }
    }

    let node_idx;

    // If this state is already in the graph ...
    if let Some(&idx) = state_node_map.get(state_index).first() {
        // If changes have occurred that might affect children, recheck this node
        if recheck[state_index.0] {
            node_idx = Some(idx);
            recheck[state_index.0] = false;
        } else {
            // Otherwise, just return the existing index
            return Some(idx);
        }
    } else {
        // This state is not yet in the graph
        // Random initial position
        let pos = Pos2::new(
            rng.random_range(-100.0..100.0),
            rng.random_range(-100.0..100.0),
        );

        let new_node_idx = graph.add_node_with_location((), pos);
        state_node_map.insert(state_index.clone(), new_node_idx);
        node_idx = Some(new_node_idx);
    }

    for child in tree.iter_ok_children(state_index) {
        // .choose_multiple(rng, rand::random_range(2..=3)) {
        let child_idx = add_to_graph(
            tree,
            &child,
            graph,
            state_node_map,
            connections,
            mask,
            recheck,
            rng,
        );
        if let Some(child_idx) = child_idx {
            let connection = (*state_index, *child);
            if let Some(_) = connections.get(&connection) {
                continue; // already added this edge
            } else {
                connections.insert(connection);
            }
            graph.add_edge(node_idx.unwrap(), child_idx, ());
        }
    }
    node_idx
}

fn prune_to_wins(tree: &Tree, nodes: &mut Vec<bool>, state: &StateIndex) -> bool {
    let mut this_one = match tree[state].result {
        Result::Win(_) => true,
        _ => false,
    };

    for child in tree.iter_ok_children(state) {
        if prune_to_wins(tree, nodes, &child) {
            this_one = true;
        }
    }

    nodes[state.0] = this_one;
    this_one
}

fn prune_to_target(tree: &Tree, mask: &mut Vec<bool>, state: &StateIndex, target: &Board) -> bool {
    let board = &tree[state].board;

    // Fast impossible-branch check
    for (_col_idx, (col, target_col)) in board.0.iter().zip(target.0.iter()).enumerate() {
        // Get non-empty pieces in both columns
        let col_pieces: Vec<_> = col.iter().filter(|&&p| p != Player::Empty).collect();
        let target_pieces: Vec<_> = target_col.iter().filter(|&&p| p != Player::Empty).collect();

        if col_pieces.len() > target_pieces.len() {
            // More pieces than target: impossible
            mask[state.0] = false;
            return false;
        }
        if col_pieces.len() == target_pieces.len() && col_pieces != target_pieces {
            // Same number but different sequence: impossible
            mask[state.0] = false;
            return false;
        }
    }

    let mut this_one = board.canonical() == target.canonical();
    for child in tree.iter_ok_children(state) {
        if prune_to_target(tree, mask, child, target) {
            this_one = true;
        }
    }
    mask[state.0] = this_one;
    this_one
}

impl Tree {
    /// Prune the tree to only nodes marked as true in `keep`, remapping indices and children.
    pub fn mask_nodes(&mut self, keep: &[bool]) {
        // Build mapping from old index to new index
        let nodes_len = self.nodes.len();
        let mut new_nodes = Vec::with_capacity(nodes_len);
        let mut new_idx = 0;
        let mut old_to_new = vec![None; nodes_len];
        for (old_idx, node) in self.nodes.drain(..).enumerate() {
            if keep[old_idx] {
                old_to_new[old_idx] = Some(new_idx);
                new_nodes.push(node); // moves, not clones
                new_idx += 1;
            }
        }
        // Update children to only include kept nodes, and remap indices
        for (new_idx, node) in new_nodes.iter_mut().enumerate() {
            let mut new_children = Vec::new();
            for child in node.ok_children() {
                if let Some(new_child_idx) = old_to_new[child.0] {
                    new_children.push(Some(StateIndex(new_child_idx)));
                }
            }
            node.children = new_children;
            // Also update node.index
            node.index = Some(StateIndex(new_idx));
        }
        // Update root_index
        if let Some(new_root) = old_to_new[self.root_index.0] {
            self.root_index = StateIndex(new_root);
        } else {
            // If root is not kept, set to 0 (empty tree)
            self.root_index = StateIndex(0);
        }
        self.nodes = new_nodes;
    }
}
