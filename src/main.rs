mod tree;

use egui::{Color32, FontId, TextFormat};
use egui_graphs::{SettingsNavigation};
use rand::seq::IndexedRandom;
use tree::{GameNode, Player, BoardKey, empty_board, find_children};
use std::{collections::HashMap, time};
use eframe::{egui::Pos2, run_native, App, NativeOptions};
use epaint::text::{LayoutJob};
use petgraph::prelude::*;
use bisetmap::BisetMap;

use crate::tree::BOARD_SIZE;

fn main() -> eframe::Result<()> {
    run_native(
        "grahpher", 
        NativeOptions::default(), 
        Box::new(|cc| Ok(Box::new(BasicApp::new(cc)))),
    )
}



pub struct BasicApp {
    g: egui_graphs::Graph,
    // game: GameNode,
    zoom_pan: bool,
    state_node_map: BisetMap<BoardKey, NodeIndex>,
    hovered_node: NodeIndex,
}

impl BasicApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut game = GameNode::from_board(empty_board(), Player::Red);
        let mut table = HashMap::new();

        let now = time::Instant::now();
        find_children(&mut game, 12, &mut table);
        println!("Tree gen took {:?}", now.elapsed());

        println!("Total unique nodes: {}", table.len());
        println!("Total children: {}", game.count_children());
        
        // Map from a unique key for each GameNode to the graph node index and its depth
        let mut state_node_map = BisetMap::new();
        let mut rng = rand::rng();

        let dig = StableDiGraph::new();
        let mut graph = egui_graphs::Graph::from(&dig);

        // Recursively add nodes and edges, reusing nodes for duplicate states, and store depth
        fn add_to_graph(
            node: &GameNode,
            graph: &mut egui_graphs::Graph,
            state_node_map: &mut BisetMap<BoardKey, NodeIndex>,
            rng: &mut impl rand::Rng,
        ) -> NodeIndex {
            let key = BoardKey(node.board.clone());
            if let Some(&idx) = state_node_map.get(&key).first() {
                return idx;
            }

            // Random initial position
            let pos = Pos2::new(
                rng.random_range(-100.0..100.0),
                rng.random_range(-100.0..100.0),
            );
            let idx = graph.add_node_with_location((), pos);
            state_node_map.insert(key, idx);
            let num = rng.random_range(1..=2);
            for child in node.children.choose_multiple(rng, num) { // Randomly select 1-2 children to cut down the number of nodes a bit
                let child_idx = add_to_graph(child, graph, state_node_map, rng);
                // Store depth as edge data (child_depth)
                graph.add_edge(idx, child_idx, ());
            }
            idx
        }

        add_to_graph(&game, &mut graph, &mut state_node_map, &mut rng);

        let binding = state_node_map.get(&BoardKey(game.board.clone()));
        let root_index = binding.first().unwrap();
        

        Self {
            g: graph,
            // game,
            zoom_pan: false,
            state_node_map,
            hovered_node: *root_index,
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


            let mut widget = egui_graphs::GraphView::<_,_,_,_,_,_,S,L>::new(&mut self.g)
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

            let hovered = self.g.hovered_node();
            if let Some(idx) = hovered {
                if idx != self.hovered_node {
                    self.hovered_node = idx;
                }
            }

            if let Some(key) = self.state_node_map.rev_get(&self.hovered_node).first() {
                ui.label(board_to_layout_job(&key.0));

            } else {
                ui.label("Hovered Node Key: None");
            }
        });
    }
}


fn board_to_layout_job(board: &[Vec<Player>]) -> LayoutJob {
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