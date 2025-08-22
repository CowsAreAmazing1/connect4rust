mod tree;
use tree::{Player, Board, GameState, StateIndex, Tree};

use egui::{Color32, FontId, TextFormat};
use egui_graphs::{SettingsNavigation};
use std::time;
use eframe::{egui::Pos2, run_native, App, NativeOptions};
use epaint::text::{LayoutJob};
use petgraph::{prelude::*};
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
    tree: Tree,
    graph: egui_graphs::Graph,
    zoom_pan: bool,
    state_node_map: BisetMap<StateIndex, NodeIndex>,
    hovered_node: NodeIndex,
}

impl BasicApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let board = Board::empty();
        // board.play(3, Player::Red);
        // board.play(2, Player::Yellow);
        // board.play(3, Player::Red);
        // board.play(3, Player::Yellow);
        // board.play(2, Player::Red);
        // board.play(4, Player::Yellow);
        // board.play(4, Player::Red);
        // board.play(2, Player::Yellow);
        // board.play(5, Player::Red);
        // board.play(5, Player::Yellow);

        
        
        
        let game = GameState::from_board(board.canonical(), Player::Red);
        let mut tree = Tree::from_root(&game);

        
        let now = time::Instant::now();
        tree.explore(1);
        println!("Tree gen took {:?}", now.elapsed());
        
        println!("Initial board:\n{}", &game);
        println!("===================================================");

        println!("Total unique nodes: {}", tree.nodes.len());
        println!("Total children: {}", tree.count_children());

        // println!("{}", tree.nodes.iter().filter(|n| !n.children.is_empty()).count());
        println!("Root: {}", tree[&tree.root]);
        println!("nodes: {}", tree.nodes.len());

        // println!("all kids: {:?}", tree.nodes.iter().map(|n| n.children.len()).collect::<Vec<_>>());
        println!("roots kids: {:?}", tree.iter_children(&tree.root));
        println!("roots first: {}", tree[tree.iter_children(&tree.root).next().unwrap()]);
        
        let mut state_node_map = BisetMap::new();
        let mut rng = rand::rng();

        let dig = StableDiGraph::new();
        let mut graph = egui_graphs::Graph::from(&dig);

    
        // Recursively add nodes and edges, reusing nodes for duplicate states, and store depth
        fn add_to_graph(
            tree: &Tree,
            state_index: &StateIndex,
            graph: &mut egui_graphs::Graph,
            state_node_map: &mut BisetMap<StateIndex, NodeIndex>,
            rng: &mut impl rand::Rng,
        ) -> NodeIndex {
            println!("called add_to_graph");
            if let Some(&idx) = state_node_map.get(state_index).first() {
                println!("Found existing node for state: {}", tree[state_index]);
                return idx;
            }

            // Random initial position
            let pos = Pos2::new(
                rng.random_range(-100.0..100.0),
                rng.random_range(-100.0..100.0),
            );
            let idx = graph.add_node_with_location((), pos);
            state_node_map.insert(state_index.clone(), idx);
            for child in tree.iter_children(state_index) {

                let child_idx = add_to_graph(tree, child, graph, state_node_map, rng);
                graph.add_edge(idx, child_idx, ());
            }
            idx
        }

        add_to_graph(&tree, &tree.root, &mut graph, &mut state_node_map, &mut rng);

        let root_node = state_node_map.get(&tree.root).first().unwrap().to_owned();

        Self {
            tree,
            graph,
            // game,
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