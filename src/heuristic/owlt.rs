use std::cell::RefCell;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::rc::Rc;
use crate::bundle::Bundle;
use crate::contact_manager::ContactManager;
use crate::heuristic::{Heuristic, HeuristicResult};
use crate::multigraph::Multigraph;
use crate::node_manager::NodeManager;
use crate::route_stage::RouteStage;
use crate::types::{Date, HopCount, NodeID};

pub struct Owlt<NM: NodeManager, CM: ContactManager>{
    target: NodeID,
    _phantom_nm: PhantomData<NM>,
    _phantom_cm: PhantomData<CM>,
}

/// Implements the Owlt-heuristic, i.e., the heuristic based on the shortest distance the current
/// node and the target node. By that, the heuristic will always the return the minimal distance
/// required to reach the target
impl <NM: NodeManager + 'static, CM: ContactManager + 'static> Heuristic<NM, CM> for Owlt<NM, CM> {
    fn new() -> Self {
        Owlt{ target: 0, _phantom_cm: PhantomData, _phantom_nm: PhantomData }
    }

    fn compute_heuristics(&mut self, route_stage: &RouteStage<NM, CM>, _bundle: &Bundle, multigraph: &Multigraph<NM, CM>) -> HeuristicResult {
        // As the remaining owlt to the target as heuristic for the at_time
        let at_time_heuristic: Date = route_stage.at_time +
            multigraph.get_minimal_delay_between_sender_and_target(route_stage.to_node, self.target);

        // Owlt has no influence on the hop count, thus we only look step into the future
        let hop_count_heuristic: HopCount = route_stage.hop_count +
            if route_stage.to_node == self.target {0} else {1};
        HeuristicResult{ at_time_heuristic, hop_count_heuristic }
    }

    fn setup(&mut self, bundle: &Bundle) {
        self.target = bundle.destinations[0]; // Only one destination so far
    }
}