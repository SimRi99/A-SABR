use std::cell::RefCell;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::rc::Rc;
use ordered_float::OrderedFloat;
use crate::bundle::Bundle;
use crate::contact_manager::ContactManager;
use crate::heuristic::{Heuristic, HeuristicDelayResult, HeuristicResult};
use crate::multigraph::Multigraph;
use crate::node_manager::NodeManager;
use crate::route_stage::RouteStage;
use crate::types::{Date, GeographicalDistance, HopCount, NodeID};

pub struct OwltCurr<NM: NodeManager, CM: ContactManager>{
    target: NodeID,
    _phantom_nm: PhantomData<NM>,
    _phantom_cm: PhantomData<CM>,
}

/// Implements the Owlt-heuristic, i.e., the heuristic based on the shortest distance the current
/// node and the target node. By that, the heuristic will always the return the minimal distance
/// required to reach the target
impl <NM: NodeManager + 'static, CM: ContactManager + 'static> Heuristic<NM, CM> for OwltCurr<NM, CM> {
    fn new() -> Self {
        OwltCurr{ target: 0, _phantom_cm: PhantomData, _phantom_nm: PhantomData }
    }

    fn compute_heuristics(&mut self, route_stage: &RouteStage<NM, CM>, _bundle: &Bundle, multigraph: &Multigraph<NM, CM>, _visited: &HashSet<NodeID>) -> HeuristicResult {
        // As the remaining owlt to the target as heuristic for the at_time
        let at_time_heuristic: Date = route_stage.at_time +
            multigraph.get_minimal_delay_between_sender_and_target(route_stage.to_node, self.target);

        // Owlt has no influence on the hop count, thus we only look step into the future
        let hop_count_heuristic: HopCount = route_stage.hop_count +
            if route_stage.to_node == self.target {0} else {1};

        // Owlt does not consider geographical distance, only the delay
        let start_time = OrderedFloat(route_stage.via.as_ref().unwrap().contact.borrow().info.start.floor());
        let cumultative_distance_heuristic: GeographicalDistance = route_stage.cumulative_distance + multigraph.get_distance_between_sender_and_target_at_time(route_stage.to_node, self.target, start_time);
        HeuristicResult{ at_time_heuristic, hop_count_heuristic, cumultative_distance_heuristic }
    }

    fn compute_delay_heuristic(&mut self, tx_node: NodeID, _bundle: &Bundle, multigraph: &Multigraph<NM, CM>, _visited: &HashSet<NodeID>) -> HeuristicDelayResult {
        HeuristicDelayResult{ heuristic_delay: multigraph.get_minimal_delay_between_sender_and_target(tx_node, self.target), heuristic_remaining_hop_count: if tx_node == self.target {0} else {1}, remaining_distance: 0.0 }
    }

    fn setup(&mut self, bundle: &Bundle) {
        self.target = bundle.destinations[0]; // Only one destination so far
    }
}