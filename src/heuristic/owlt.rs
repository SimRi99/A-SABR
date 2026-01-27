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

pub struct Owlt<NM: NodeManager, CM: ContactManager>{
    multigraph: Rc<RefCell<Multigraph<NM, CM>>>,
    target: NodeID
}

/// Implements the Owlt-heuristic, i.e., the heuristic based on the shortest distance the current
/// node and the target node. By that, the heuristic will always the return the minimal distance
/// required to reach the target
impl <NM: NodeManager + 'static, CM: ContactManager + 'static> Heuristic<NM, CM> for Owlt<NM, CM> {
    fn new(multigraph: Rc<RefCell<Multigraph<NM, CM>>>, target: NodeID) -> Box<dyn Heuristic<NM, CM>> {
        Box::new(Owlt{ multigraph, target })
    }

    fn compute_at_time_heuristic(
        &mut self,
        route_stage: &RouteStage<NM, CM>, _bundle: &Bundle
    ) -> Date {
        route_stage.at_time +
            self.multigraph.borrow().
                get_minimal_delay_between_sender_and_target(route_stage.to_node, self.target)
    }

    fn compute_hop_count_heuristic(&mut self, route_stage: &RouteStage<NM, CM>, _bundle: &Bundle) -> HopCount {
        route_stage.hop_count + if route_stage.to_node == self.target {0} else {1}
    }

    fn set_visited_set(&mut self, _visited: Rc<RefCell<HashSet<NodeID>>>) {
        // Do nothing, is not used here.
    }
}