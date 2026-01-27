mod zero;
mod owlt;
mod k_look_ahead;

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use crate::bundle::Bundle;
use crate::contact_manager::ContactManager;
use crate::multigraph::Multigraph;
use crate::node_manager::NodeManager;
use crate::route_stage::RouteStage;
use crate::types::{Date, HopCount, NodeID};

/// A trait for defining heuristic functions.
///
/// A heuristic function allows to predict for given route stage the remaining cost metrics in order
/// to reach the target of the bundle. As of now, supported cost metrics are the at_time and the hop count.
/// All heuristics need the multigraph as well as the target id in order to be instantiated.
///
/// # Type Parameters
///
/// * `NM` - A generic type that implements the `NodeManager` trait.
/// * `CM` - A generic type that implements the `ContactManager` trait.

pub trait Heuristic<NM: NodeManager, CM: ContactManager> {
    /// Creates a new instance of the heuristic function with the provided multigraph and target id.
    ///
    /// # Parameters
    ///
    /// * `multigraph` - A pointer to the multigraph.
    /// * `target_id` - The node id of the target.
    ///
    /// # Returns
    ///
    /// A new instance of the struct implementing `Heuristic`.
    fn new(multigraph: Rc<RefCell<Multigraph<NM, CM>>>, target: NodeID) -> Box<dyn Heuristic<NM, CM>> where Self: Sized;


    /// Predicts the heuristic_at_time for a given route stage, i.e., the estimated at_time
    /// when the bundle will be received at its target.
    ///
    /// # Parameters
    ///
    /// * `route_stage` - A reference to the `RouteStage` whose remaining distance has to be measured.
    ///
    /// # Returns
    ///
    /// * `Date` - The date at which the bundle is received.
    fn compute_at_time_heuristic(
        &mut self,
        route_stage: &RouteStage<NM, CM>,
        bundle: &Bundle
    ) -> Date;

    /// Predict the predicate hop counts for a `RouteStage`,
    /// i.e., the predicated amount of hops a bundle has to make before reaching its target.
    ///
    /// # Parameters
    /// * `route_stage` - A reference to the `RouteStage` whose remaining hop_count has to be measured.
    ///
    /// # Returns
    ///
    /// * `HopCount` - The hop count how many hop_counts have to be made in order the target.
    fn compute_hop_count_heuristic(&mut self, route_stage: &RouteStage<NM, CM>, bundle: &Bundle) -> HopCount;

    /// Sets a hashset of visited nodes, which is relevant for certain heursitics, e.g. the k_look_ahead
    /// heuristic
    ///
    /// # Parameters
    /// * `visited` - A shared set of visited nodes.
    fn set_visited_set(&mut self, visited: Rc<RefCell<HashSet<NodeID>>>);
}