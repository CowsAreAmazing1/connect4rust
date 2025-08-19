mod tree;

use egui::Color32;
use egui_graphs::{DefaultNodeShape, Node, SettingsStyle};
use tree::{GameNode, Player, empty_board, find_children};
use std::collections::HashMap;
use eframe::{egui::Pos2, run_native, App, NativeOptions};
use petgraph::prelude::*;

use crate::tree::BoardKey;

fn main() -> eframe::Result {
    run_native(
        "grahpher", 
        NativeOptions::default(), 
        Box::new(|cc| Ok(Box::new(BasicApp::new(cc)))),
    )
}



pub struct BasicApp {
    g: egui_graphs::Graph,
}

impl BasicApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut game = GameNode::from_board(empty_board(), Player::Red);
        let mut table = HashMap::new();
        find_children(&mut game, 6, &mut table);
        println!("Total unique nodes: {}", table.len());
        println!("Total children: {}", game.count_children());
        
        // Map from a unique key for each GameNode to the graph node index and its depth
        let mut state_to_node: HashMap<BoardKey, (NodeIndex, usize)> = HashMap::new();
        let mut rng = rand::rng();

        let dig = StableDiGraph::new();
        let mut graph = egui_graphs::Graph::from(&dig);

        // Helper to count non-empty positions (turns played)
        fn count_turns(board: &Vec<Vec<tree::Player>>) -> usize {
            board.iter().flatten().filter(|&&p| p != tree::Player::Empty).count()
        }

        // Recursively add nodes and edges, reusing nodes for duplicate states, and store depth
        fn add_to_graph(
            node: &GameNode,
            graph: &mut egui_graphs::Graph,
            state_to_node: &mut HashMap<BoardKey, (NodeIndex, usize)>,
            rng: &mut impl rand::Rng,
            depth: usize,
        ) -> NodeIndex {
            let key = BoardKey(node.board.clone());
            if let Some(&(idx, _)) = state_to_node.get(&key) {
                return idx;
            }
            // Random initial position
            let pos = Pos2::new(
                rng.random_range(-100.0..100.0),
                rng.random_range(-100.0..100.0),
            );
            // Store depth as node data
            let graph_node = Node::<(), (), Directed, u32, DefaultNodeShape>::new(()).set_color(Color32::RED);
            let idx = graph.add_node_with_location(graph_node, pos);
            state_to_node.insert(key, (idx, depth));
            for child in &node.children {
                let child_depth = count_turns(&child.board);
                let child_idx = add_to_graph(child, graph, state_to_node, rng, child_depth);
                // Store depth as edge data (child_depth)
                graph.add_edge(idx, child_idx, ());
            }
            idx
        }

        let root_depth = count_turns(&game.board);
        add_to_graph(&game, &mut graph, &mut state_to_node, &mut rng, root_depth);

        Self { g: graph }
    }
}

impl App for BasicApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // let mut widget = egui_graphs::DefaultGraphView::new(&mut self.g);
            // ui.add(&mut widget);

            type L = egui_graphs::LayoutForceDirected<egui_graphs::FruchtermanReingoldWithCenterGravity>;
            type S = egui_graphs::FruchtermanReingoldWithCenterGravityState;


            let mut widget = egui_graphs::GraphView::<_,_,_,_,_,_,S,L>::new(&mut self.g);
            ui.add(&mut widget);

            // // Force‑Directed (FR) #with Center Gravity
            // type L = egui_graphs::LayoutForceDirected<egui_graphs::FruchtermanReingoldWithCenterGravity>;
            // type S = egui_graphs::FruchtermanReingoldWithCenterGravityState;
            // let mut view = egui_graphs::GraphView::<_,_,_,_,_,_,S,L>::new(&mut self.g);
            // ui.add(&mut view);
        });
    }
}
