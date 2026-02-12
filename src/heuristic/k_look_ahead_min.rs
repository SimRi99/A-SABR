
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use crate::contact_manager::ContactManager;
use crate::heuristic::{Heuristic, HeuristicDelayResult, HeuristicResult};
use crate::multigraph::Multigraph;
use crate::node_manager::NodeManager;
use crate::route_stage::RouteStage;
use crate::types::{Duration, HopCount, NodeID};
use crate::bundle::Bundle;

pub struct KLookAheadMin<NM: NodeManager, CM: ContactManager, H: Heuristic<NM, CM>> {
    target: NodeID,
    k: usize,
    final_heuristic: H,
    visited: HashSet<NodeID>,
    table: HashMap<NodeID, HashMap<usize, HeuristicDelayResult>>,
    _phantom_nm: PhantomData<NM>,
    _phantom_cm: PhantomData<CM>,
}

/// Implements the K-Look-Ahead-heuristic. This heuristic will not only evaluate the best possible
/// node with its owlt heuristic, but also evaluate the lowest costs of all reaching all neighbors
/// within steps, plus another, final heuristic to reach the target from the neighbor.
/// By that, the heuristic balances accuracy of the heuristic and computational overhead.
/// Note that for k=0, the heuristic is equal to the owlt heuristic, and for k=inf, it is equal to
/// h*.

impl<NM: NodeManager, CM: ContactManager, H: Heuristic<NM, CM>> KLookAheadMin<NM, CM, H> {
    fn compute_for_k(&mut self, k: usize, tx_node: NodeID, bundle: &Bundle, multigraph: &Multigraph<NM, CM>, visited: &HashSet<NodeID>) {

        if k==0 {
            let heuristic_delay_result: HeuristicDelayResult = self.final_heuristic.compute_delay_heuristic(tx_node, bundle, multigraph, visited);
            self.add_to_table(tx_node, k, heuristic_delay_result);
            return;
        }
        if tx_node == self.target {
            let heuristic_delay_result: HeuristicDelayResult = HeuristicDelayResult {heuristic_delay: 0.0, heuristic_remaining_hop_count: 0};
            self.add_to_table(tx_node, k, heuristic_delay_result);
            return;
        }

        let mut lowest_delay: HeuristicDelayResult = HeuristicDelayResult {heuristic_delay: Duration::MAX, heuristic_remaining_hop_count: HopCount::MAX};

        self.add_node_to_visited(tx_node);
        let sender =  &multigraph.senders[tx_node as usize];

        for receiver in &sender.receivers {
            let receiver_id: NodeID = receiver.node.borrow().info.id;
            if self.contains_node(receiver_id) || visited.contains(&receiver_id) {
                continue;
            }
            self.add_node_to_visited(receiver_id);
            if self.get_from_table(receiver_id, k - 1).is_none() {
                self.compute_for_k(k - 1, receiver_id, bundle, multigraph, visited);
            }
            let receiver_delay = self.get_from_table(receiver_id, k - 1).unwrap();
            let node_heuristic_delay = receiver_delay.heuristic_delay + multigraph.get_minimal_delay_between_sender_and_target(tx_node, receiver_id);
            let node_heuristic_hop_count = receiver_delay.heuristic_remaining_hop_count + 1;
            if node_heuristic_delay < lowest_delay.heuristic_delay {
                lowest_delay.heuristic_delay = node_heuristic_delay;
                lowest_delay.heuristic_remaining_hop_count = node_heuristic_hop_count
            } else if node_heuristic_delay == lowest_delay.heuristic_delay && node_heuristic_hop_count < lowest_delay.heuristic_remaining_hop_count {
                lowest_delay.heuristic_delay = node_heuristic_delay;
                lowest_delay.heuristic_remaining_hop_count = node_heuristic_hop_count;
            }
            self.remove_node_from_visited(receiver_id);
        }
        self.remove_node_from_visited(tx_node);
        self.add_to_table(tx_node, k, lowest_delay);
    }


    fn add_to_table(&mut self, tx_node: NodeID, k: usize, result: HeuristicDelayResult) {
        if !self.table.contains_key(&tx_node) {
            self.table.insert(tx_node, HashMap::new());
        }
        self.table.get_mut(&tx_node).unwrap().insert(k, result);
    }

    fn get_from_table(&self, tx_node: NodeID, k: usize) -> Option<&HeuristicDelayResult> {
        if let Some(map) = self.table.get(&tx_node) {
            if let Some(result) = map.get(&k) {
                return Some(&result);
            }
            return None;
        }
        return None;
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

impl <NM: NodeManager +'static, CM: ContactManager +'static, H: Heuristic<NM, CM> > Heuristic<NM, CM> for KLookAheadMin<NM, CM, H> {
    fn new() -> Self {
        KLookAheadMin {target: 0, k: 3, final_heuristic: H::new(), table: HashMap::new(), visited: HashSet::new(), _phantom_cm: PhantomData, _phantom_nm: PhantomData}
    }

    fn compute_heuristics(&mut self, route_stage: &RouteStage<NM, CM>, bundle: &Bundle, multigraph: &Multigraph<NM, CM>, visited: &HashSet<NodeID>) -> HeuristicResult {
        // Compute the best heuristic
        if self.get_from_table(route_stage.to_node, self.k).is_none() {
            self.compute_for_k(self.k, route_stage.to_node, bundle, multigraph, visited);
        }

        let lowest_delay = self.get_from_table(route_stage.to_node, self.k).unwrap();

        // Extract the result
        let result: HeuristicResult = HeuristicResult{at_time_heuristic: route_stage.at_time + lowest_delay.heuristic_delay, hop_count_heuristic: route_stage.hop_count + lowest_delay.heuristic_remaining_hop_count};

        result
    }

    fn compute_delay_heuristic(&mut self, tx_node: NodeID, bundle: &Bundle, multigraph: &Multigraph<NM, CM>, visited: &HashSet<NodeID>) -> HeuristicDelayResult {
        // Compute the best heuristic
        if self.get_from_table(tx_node, self.k).is_none() {
            self.compute_for_k(self.k, tx_node, bundle, multigraph, visited);
        }

        let lowest_delay = self.get_from_table(tx_node, self.k).unwrap();
        // Extract the result
        let result: HeuristicDelayResult = HeuristicDelayResult{heuristic_delay : lowest_delay.heuristic_delay, heuristic_remaining_hop_count: lowest_delay.heuristic_remaining_hop_count};

        result
    }

    fn setup(&mut self, bundle: &Bundle) {
        self.final_heuristic.setup(bundle);
        self.target = bundle.destinations[0];
    }

}