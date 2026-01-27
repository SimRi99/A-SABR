use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use crate::bundle::Bundle;
use crate::contact_manager::ContactManager;
use crate::heuristic::Heuristic;
use crate::multigraph::Multigraph;
use crate::node_manager::NodeManager;
use crate::route_stage::RouteStage;
use crate::types::{Date, HopCount, NodeID};

pub struct Zero{}

/// Implements the Zero-heuristic, i.e., the heuristic always returning 0. In other words, this
/// heuristic will lead for A* to become the regular Dijkstra algorithm with no heuristic
impl <NM: NodeManager, CM: ContactManager> Heuristic<NM, CM> for Zero {
    fn new(_multigraph: Rc<RefCell<Multigraph<NM, CM>>>, _target: NodeID) -> Box<dyn Heuristic<NM, CM>> {
        Box::new(Zero{})
    }

    fn compute_at_time_heuristic(
        &mut self,
        route_stage: &RouteStage<NM, CM>, _bundle: &Bundle
    ) -> Date {
        route_stage.at_time
    }

    fn compute_hop_count_heuristic(&mut self, route_stage: &RouteStage<NM, CM>, _bundle: &Bundle) -> HopCount {
        route_stage.hop_count
    }

    fn set_visited_set(&mut self, _visited: Rc<RefCell<HashSet<NodeID>>>) {
        // Do nothing, is not used here
    }
}