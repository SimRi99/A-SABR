pub(crate) mod zero;
pub(crate) mod owlt;
pub(crate) mod k_look_ahead;
pub(crate) mod k_look_ahead_min;

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use crate::bundle::Bundle;
use crate::contact_manager::ContactManager;
use crate::multigraph::Multigraph;
use crate::node_manager::NodeManager;
use crate::route_stage::RouteStage;
use crate::types::{Date, Duration, HopCount, NodeID};


pub struct HeuristicResult {
    // A heuristic value for the at_time
    pub at_time_heuristic: Date,

    // A heuristic value for the hop count
    pub hop_count_heuristic: HopCount
}

pub struct HeuristicDelayResult {
    pub heuristic_delay: Duration,
    pub heuristic_remaining_hop_count: HopCount
}

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
    /// Creates a new instance of the heuristic function with the provided multigraph.
    ///
    /// # Parameters
    ///
    /// * `multigraph` - A pointer to the multigraph.
    ///
    /// # Returns
    ///
    /// A new instance of the struct implementing `Heuristic`.
    fn new() -> Self;

    /// Compute all relevant heuristic values required for a heuristic result, e.g., the at_time
    /// heuristics, or the hop count heuristic.
    ///
    /// # Parameters
    ///
    /// * `route_stage` - A reference to the `RouteStage` whose remaining distance has to be measured.
    /// * `bundle` - A reference to the `Bundle` to be routed.
    ///
    /// # Returns
    ///
    /// * `HeuristicResult` - The results of the heuristic computations.
    fn compute_heuristics(&mut self, route_stage: &RouteStage<NM, CM>, bundle: &Bundle, multigraph: &Multigraph<NM, CM>, visited: &HashSet<NodeID>) -> HeuristicResult;

    fn compute_delay_heuristic(&mut self, tx_node: NodeID, bundle: &Bundle, multigraph: &Multigraph<NM, CM>, visited: &HashSet<NodeID>) -> HeuristicDelayResult;

    /// Sets up the heuristic with the bundle, as well as a shared visited set, such that certain
    /// heuristics can keep track of which nodes has been traversed already
    ///
    /// # Parameters
    /// * `visited` - A shared set of visited nodes.
    /// * `visited` - Optionally, a shared reference to a hashset if nodes should be collected
    fn setup(&mut self, bundle: &Bundle);
}