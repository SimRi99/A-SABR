use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use crate::bundle::Bundle;
use crate::contact_manager::ContactManager;
use crate::heuristic::{Heuristic, HeuristicDelayResult, HeuristicResult};
use crate::multigraph::Multigraph;
use crate::node_manager::NodeManager;
use crate::route_stage::RouteStage;
use crate::types::{NodeID};

pub struct Zero{}

/// Implements the Zero-heuristic, i.e., the heuristic always returning 0. In other words, this
/// heuristic will lead for A* to become the regular Dijkstra algorithm with no heuristic
impl <NM: NodeManager, CM: ContactManager> Heuristic<NM, CM> for Zero {
    fn new() -> Self {
        Zero{}
    }

    fn compute_heuristics(&mut self, route_stage: &RouteStage<NM, CM>, _bundle: &Bundle, _multigraph: &Multigraph<NM, CM>, _visited: &HashSet<NodeID>) -> HeuristicResult {
        HeuristicResult{ at_time_heuristic: route_stage.at_time, hop_count_heuristic: route_stage.hop_count }
    }

    fn compute_delay_heuristic(&mut self, _tx_node: NodeID, _bundle: &Bundle, _multigraph: &Multigraph<NM, CM>, _visited: &HashSet<NodeID>) -> HeuristicDelayResult {
        HeuristicDelayResult{ heuristic_delay: 0.0, heuristic_remaining_hop_count: 0 }
    }

    fn setup(&mut self, _bundle: &Bundle) {
        // Do nothing, is not used here
    }
}