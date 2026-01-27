use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use crate::contact_manager::ContactManager;
use crate::heuristic::Heuristic;
use crate::heuristic::owlt::Owlt;
use crate::multigraph::Multigraph;
use crate::node_manager::NodeManager;
use crate::route_stage::RouteStage;
use crate::types::{Date, HopCount, NodeID};
use crate::pathfinding::{try_make_hop};
use crate::bundle::Bundle;

pub struct KLookAhead<NM: NodeManager, CM: ContactManager> {
    multigraph: Rc<RefCell<Multigraph<NM, CM>>>,
    target: NodeID,
    k: usize,
    final_heuristic: Box<dyn Heuristic<NM, CM>>,
    visited: Option<Rc<RefCell<HashSet<NodeID>>>>,
    best_date: Date
}

/// Implements the K-Look-Ahead-heuristic. This heuristic will not only evaluate the best possible
/// node with its owlt heuristic, but also evaluate the lowest costs of all reaching all neighbors
/// within steps, plus another, final heuristic to reach the target from the neighbor.
/// By that, the heuristic balances accuracy of the heuristic and computational overhead.
/// Note that for k=0, the heuristic is equal to the owlt heuristic, and for k=inf, it is equal to
/// h*.

impl<NM: NodeManager, CM: ContactManager> KLookAhead<NM, CM> {
    fn compute_for_k(&mut self, k: usize, route_stage: &RouteStage<NM, CM>, bundle: &Bundle) {
        if k==0 {
            let possible_date: Date = self.final_heuristic.compute_at_time_heuristic(&route_stage, bundle);
            self.check_for_best_date(possible_date);
        }
        if route_stage.to_node == self.target {
           self.check_for_best_date(route_stage.at_time)
        }

        let route_stage_ref =
            Rc::new(RefCell::new(route_stage.clone_work_area()));
        let tx_node_id = route_stage.to_node;


        let multigraph_rc = Rc::clone(&self.multigraph);
        let sender = &multigraph_rc.borrow().senders[tx_node_id as usize];

        for receiver in &sender.receivers {
            let receiver_id: NodeID = receiver.node.borrow().info.id;
            if self.contains_node(receiver_id) {
                continue;
            }
            self.add_node_to_visited(receiver_id);
            if let Some(first_contact_index) =
                receiver.get_first_idx(route_stage.at_time)
            {
                if let Some(route_proposition) = try_make_hop(
                    first_contact_index,
                    &route_stage_ref,
                    bundle,
                    &receiver.contacts_to_receiver,
                    &sender.node,
                    &receiver.node,
                    &None
                ) {
                    self.compute_for_k(k - 1, &route_proposition, bundle);
                }
            }
            self.remove_node_from_visited(receiver_id);
        }
    }

    // Check whether a newly found date might be the best one
    fn check_for_best_date(&mut self, new_date: Date) {
        if new_date < self.best_date {
            self.best_date = new_date
        }
    }

    // Check whether the current node was already visited
    fn contains_node(&self, node: NodeID) -> bool {
        if let Some(visited_rc) = &self.visited {
            return visited_rc.borrow().contains(&node);
        }
        false
    }

    // Add a new node to the visited set
    fn add_node_to_visited(&mut self, node: NodeID) -> bool {
        if let Some(visited_rc) = &self.visited {
            visited_rc.borrow_mut().insert(node);
        }
        self.visited = Some(Rc::new(RefCell::new(HashSet::new())));
        if let Some(visited_rc) = &self.visited {
            visited_rc.borrow_mut().insert(node);
        }
        false

    }

    // Remove node from the visited set
    fn remove_node_from_visited(&mut self, node: NodeID) -> bool {
        if let Some(visited_rc) = &self.visited {
            visited_rc.borrow_mut().remove(&node);
        }
        true
    }
}

impl <NM: NodeManager +'static, CM: ContactManager +'static> Heuristic<NM, CM> for KLookAhead<NM, CM> {
    fn new(multigraph: Rc<RefCell<Multigraph<NM, CM>>>, target: NodeID) -> Box<dyn Heuristic<NM, CM>> {
       Box::new(KLookAhead { multigraph: Rc::clone(&multigraph), target, k: 2, final_heuristic: Owlt::new(Rc::clone(&multigraph), target), visited: None, best_date: Date::MAX})
    }

    fn compute_at_time_heuristic(
        &mut self,
        route_stage: &RouteStage<NM, CM>, bundle: &Bundle
    ) -> Date {
        self.compute_for_k(self.k, route_stage, bundle);
        let best_date_heuristic: Date = self.best_date;

        // Reset for next run
        self.best_date = Date::MAX;

        best_date_heuristic
    }

    fn compute_hop_count_heuristic(&mut self, route_stage: &RouteStage<NM, CM>, bundle: &Bundle) -> HopCount {
        self.final_heuristic.compute_hop_count_heuristic(route_stage, bundle)
    }

    fn set_visited_set(&mut self, visited: Rc<RefCell<HashSet<NodeID>>>) {
        self.visited = Some(Rc::clone(&visited));
    }

}