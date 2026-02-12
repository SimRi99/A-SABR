use std::cell::RefCell;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::rc::Rc;
use crate::contact_manager::ContactManager;
use crate::heuristic::{Heuristic, HeuristicDelayResult, HeuristicResult};
use crate::multigraph::Multigraph;
use crate::node_manager::NodeManager;
use crate::route_stage::RouteStage;
use crate::types::{Date, HopCount, NodeID};
use crate::pathfinding::{try_make_hop};
use crate::bundle::Bundle;

pub struct KLookAhead<NM: NodeManager, CM: ContactManager, H: Heuristic<NM, CM>> {
    target: NodeID,
    k: usize,
    final_heuristic: H,
    visited: HashSet<NodeID>,
    best_at_time: Date,
    best_hop_count: HopCount,
    _phantom_nm: PhantomData<NM>,
    _phantom_cm: PhantomData<CM>,
}

/// Implements the K-Look-Ahead-heuristic. This heuristic will not only evaluate the best possible
/// node with its owlt heuristic, but also evaluate the lowest costs of all reaching all neighbors
/// within steps, plus another, final heuristic to reach the target from the neighbor.
/// By that, the heuristic balances accuracy of the heuristic and computational overhead.
/// Note that for k=0, the heuristic is equal to the owlt heuristic, and for k=inf, it is equal to
/// h*.

impl<NM: NodeManager, CM: ContactManager, H: Heuristic<NM, CM>> KLookAhead<NM, CM, H> {
    fn compute_for_k(&mut self, k: usize, route_stage: &RouteStage<NM, CM>, bundle: &Bundle, multigraph: &Multigraph<NM, CM>, visited: &HashSet<NodeID>) {
        if k==0 {
            let heuristic_result: HeuristicResult = self.final_heuristic.compute_heuristics(route_stage, bundle, multigraph, visited);
            self.check_for_best_date(heuristic_result.at_time_heuristic);
            self.check_for_best_hop_count(heuristic_result.hop_count_heuristic);
            return;
        }
        if route_stage.to_node == self.target {
            self.check_for_best_date(route_stage.at_time);
            self.check_for_best_hop_count(route_stage.hop_count_heuristic);
            return;
        }

        let route_stage_ref =
            Rc::new(RefCell::new(route_stage.clone_work_area()));
        let tx_node_id = route_stage.to_node;

        self.add_node_to_visited(tx_node_id);
        let sender =  &multigraph.senders[tx_node_id as usize];

        for receiver in &sender.receivers {
            let receiver_id: NodeID = receiver.node.borrow().info.id;
            if self.contains_node(receiver_id) || visited.contains(&receiver_id) {
                continue;
            }
            self.add_node_to_visited(receiver_id);
            if let Some(first_contact_index) =
                receiver.get_first_idx(route_stage.at_time)
            {
                if let Some(route_proposition) = try_make_hop::<NM, CM, H>(
                    first_contact_index,
                    &route_stage_ref,
                    bundle,
                    &receiver.contacts_to_receiver,
                    &sender.node,
                    &receiver.node,
                    None,
                    multigraph,
                    visited
                ) {
                    self.compute_for_k(k - 1, &route_proposition, bundle, multigraph, visited);
                }
            }
            self.remove_node_from_visited(receiver_id);
        }
        self.remove_node_from_visited(tx_node_id);
    }

    // Check whether a newly found date might be the best one
    fn check_for_best_date(&mut self, new_date: Date) {
        if new_date < self.best_at_time {
            self.best_at_time = new_date
        }
    }

    // Check whether a newly found hop count might be the best one
    fn check_for_best_hop_count(&mut self, new_hop_count: HopCount) {
        if new_hop_count < self.best_hop_count {
            self.best_hop_count = new_hop_count;
        }
    }

    // Check whether the current node was already visited
    fn contains_node(&self, node: NodeID) -> bool {
        self.visited.contains(&node)
    }

    // Add a new node to the visited set
    fn add_node_to_visited(&mut self, node: NodeID) {
        self.visited.insert(node);
    }

    // Remove node from the visited set
    fn remove_node_from_visited(&mut self, node: NodeID) {
        self.visited.remove(&node);
    }
}

impl <NM: NodeManager +'static, CM: ContactManager +'static, H: Heuristic<NM, CM> > Heuristic<NM, CM> for KLookAhead<NM, CM, H> {
    fn new() -> Self {
       KLookAhead {target: 0, k: 1, final_heuristic: H::new(), visited: HashSet::new(), best_at_time: Date::MAX, best_hop_count: HopCount::MAX, _phantom_cm: PhantomData, _phantom_nm: PhantomData}
    }

    fn compute_heuristics(&mut self, route_stage: &RouteStage<NM, CM>, bundle: &Bundle, multigraph: &Multigraph<NM, CM>, visited: &HashSet<NodeID>) -> HeuristicResult {
        // Compute the best heuristic
        self.compute_for_k(self.k, route_stage, bundle, multigraph, visited);

        // Extract the result
        let result: HeuristicResult = HeuristicResult{at_time_heuristic: self.best_at_time, hop_count_heuristic: self.best_hop_count};

        // Reset values for next computation
        self.best_at_time = Date::MAX;
        self.best_hop_count = HopCount::MAX;

        result
    }
    fn compute_delay_heuristic(&mut self, tx_node: NodeID, bundle: &Bundle, multigraph: &Multigraph<NM, CM>, visited: &HashSet<NodeID>) -> HeuristicDelayResult {
        // not supported right now
        self.final_heuristic.compute_delay_heuristic(tx_node, bundle, multigraph, visited)
    }

    fn setup(&mut self, bundle: &Bundle) {
        self.final_heuristic.setup(bundle);
        self.target = bundle.destinations[0];
    }

}