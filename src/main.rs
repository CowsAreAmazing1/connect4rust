mod tree;
use tree::{Player, Board, Result, GameState, StateIndex, Tree};

use egui::{Color32, FontId, TextFormat};
use egui_graphs::{SettingsNavigation};
use std::time;
use eframe::{egui::Pos2, run_native, App, NativeOptions};
use epaint::text::{LayoutJob};
use petgraph::{prelude::*};
use bisetmap::BisetMap;
use rand::prelude::*;

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
    zoom_pan: bool,
    state_node_map: BisetMap<StateIndex, NodeIndex>,
    hovered_node: NodeIndex,
}

impl BasicApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut board = Board::empty();

        board.play(3, Player::Red);
        board.play(2, Player::Yellow);
        board.play(3, Player::Red);
        board.play(3, Player::Yellow);
        board.play(2, Player::Red);
        board.play(4, Player::Yellow);
        board.play(4, Player::Red);
        board.play(2, Player::Yellow);
        board.play(5, Player::Red);
        board.play(5, Player::Yellow);
        
        
        let game = GameState::from_board(board.canonical(), Player::Red);
        let mut tree = Tree::from_root(&game);


        
        let now = time::Instant::now();
        tree.explore(11);

        println!("Tree gen took {:?}", now.elapsed());
        
        println!("Total unique nodes: {}", tree.nodes.len());
        println!("Total children: {}", tree.count_children());

        println!("{}", &tree[&tree.root_index]);


        let mut win_nodes = vec![false; tree.nodes.len()];
        prune_to_wins(&tree, &mut win_nodes, &tree.root_index);


        println!("Pruned to {} nodes", &win_nodes.iter().filter(|&n| *n).count());
        tree.prune_to_win_nodes(&win_nodes);
        println!("Pruning took {:?}", now.elapsed());
        
        
        let mut state_node_map = BisetMap::new();
        let mut rng = rand::rng();


        let dig = StableDiGraph::new();
        let mut graph = egui_graphs::Graph::from(&dig);

    
        // Recursively add nodes and edges, reusing nodes for duplicate states
        fn add_to_graph(
            tree: &Tree,
            state_index: &StateIndex,
            graph: &mut egui_graphs::Graph,
            state_node_map: &mut BisetMap<StateIndex, NodeIndex>,
            rng: &mut impl rand::Rng,
        ) -> NodeIndex {
            if let Some(&idx) = state_node_map.get(state_index).first() {
                // println!("Found existing node for state: {}", tree[state_index]);
                return idx;
            }

            // Random initial position
            let pos = Pos2::new(
                rng.random_range(-100.0..100.0),
                rng.random_range(-100.0..100.0),
            );
            let idx = graph.add_node_with_location((), pos);
            state_node_map.insert(state_index.clone(), idx);
            for child in tree.iter_children(state_index).choose_multiple(rng, rand::random_range(1..=3)) {
                let child_idx = add_to_graph(tree, child, graph, state_node_map, rng);
                graph.add_edge(idx, child_idx, ());
            }
            idx
        }

        let now = time::Instant::now();
        add_to_graph(&tree, &tree.root_index, &mut graph, &mut state_node_map, &mut rng);
        println!("Graph generating took {:?}", now.elapsed());


        let root_node = state_node_map.get(&tree.root_index).first().unwrap().to_owned();

        Self {
            tree,
            graph,
            zoom_pan: false,
            state_node_map,
            hovered_node: root_node,
        }
    }
}

impl App for BasicApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // let mut widget = egui_graphs::DefaultGraphView::new(&mut self.g);
            // ui.add(&mut widget);

            type L = egui_graphs::LayoutForceDirected<egui_graphs::FruchtermanReingoldWithCenterGravity>;
            type S = egui_graphs::FruchtermanReingoldWithCenterGravityState;


            let mut widget = egui_graphs::GraphView::<_,_,_,_,_,_,S,L>::new(&mut self.graph)
                .with_navigations(&SettingsNavigation::new()
                    .with_zoom_and_pan_enabled(self.zoom_pan)
                    .with_fit_to_screen_enabled(!self.zoom_pan)
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

            // ui.horizontal_centered(|ui| {
            //         ui.vertical_centered(|ui| {
            //             let texter = self.game.children[1].children[2].children[1].children[0].children[0].to_string();
            //             ui.label(egui::RichText::new(texter).size(20.0).color(Color32::RED));
            //         })
            //     }
            // )

            let hovered = self.graph.hovered_node();
            if let Some(idx) = hovered {
                if idx != self.hovered_node {
                    self.hovered_node = idx;
                }
            }

            if let Some(key) = self.state_node_map.rev_get(&self.hovered_node).first() {
                ui.label(board_to_layout_job(self.tree.get_board(key)));

            } else {
                ui.label("Hovered Node Key: None");
            }
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
                TextFormat::simple(FontId::monospace(font_size), color)
            );
        }
        job.append(
            "\n",
            0.0,
            TextFormat::simple(FontId::monospace(font_size), Color32::BLACK)
        );
    }
    
    job
}









fn prune_to_wins(tree: &Tree, nodes: &mut Vec<bool>, state: &StateIndex) -> bool {
    let mut this_one = match tree[state].result {
        Result::Win(_) => true,
        _              => false,
    };

    for child in tree.iter_children(state) {
        if prune_to_wins(tree, nodes, child) {
            this_one = true;
        }
    }
    
    nodes[state.0] = this_one;
    this_one
}


impl Tree {
    /// Prune the tree to only nodes marked as true in `keep`, remapping indices and children.
    pub fn prune_to_win_nodes(&mut self, keep: &[bool]) {
        // Build mapping from old index to new index
        let nodes_len = self.nodes.len();
        let mut new_nodes = Vec::with_capacity(nodes_len);
        let mut old_to_new = vec![None; nodes_len];
        for (old_idx, node) in self.nodes.drain(..).enumerate() {
            if keep[old_idx] {
                let new_idx = new_nodes.len();
                old_to_new[old_idx] = Some(new_idx);
                new_nodes.push(node); // moves, not clones
            }
        }
        // Update children to only include kept nodes, and remap indices
        for (new_idx, node) in new_nodes.iter_mut().enumerate() {
            let mut new_children = Vec::new();
            for child in &node.children {
                if let Some(&Some(new_child_idx)) = old_to_new.get(child.0) {
                    new_children.push(crate::tree::StateIndex(new_child_idx));
                }
            }
            node.children = new_children;
            // Also update node.index
            node.index = Some(crate::tree::StateIndex(new_idx));
        }
        // Update root_index
        if let Some(new_root) = old_to_new[self.root_index.0] {
            self.root_index = crate::tree::StateIndex(new_root);
        } else {
            // If root is not kept, set to 0 (empty tree)
            self.root_index = crate::tree::StateIndex(0);
        }
        self.nodes = new_nodes;
    }
}