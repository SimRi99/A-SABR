use std::cmp::Ordering;

use crate::{
    contact_manager::ContactManager, node_manager::NodeManager,
    pathfinding::hybrid_parenting::HybridParentingOrd, route_stage::RouteStage,
};
use super::{Distance};

/// A struct allowing to use the Schedule-Aware Bundle Routing distance definition, but using
/// heuristics instead in addition of actual measured distances.
///
/// `HeuristicSabr` is used to implement the `Distance` trait, providing a comparison method
/// for determining the order of `RouteStage` instances based on a set of criteria
/// (such as `at_time_heuristic` (i.e. arrival time), `hop_count_heuristic`, and `expiration`).
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct HeuristicSabr {}

impl<NM: NodeManager, CM: ContactManager> Distance<NM, CM> for HeuristicSabr {
    /// Compares two `RouteStage` instances to determine their ordering based on
    /// the SABR standard tie-break rules, adapted to A*-based heuristics.
    ///
    /// The comparison follows these rules, in descending order of priority:
    /// 1. `at_time_heuristic`: The `RouteStage` with a later `at_time_heuristic` is considered greater.
    /// 2. `hop_count_heuristic`: If `at_time_heuristic` is equal, the one with a higher `hop_count_heuristic` is greater.
    /// 3. `expiration`: If both `at_time_heuristic` and `hop_count_heuristic` are equal, the one with a lower `expiration` is greater.
    /// Note that for 3., no heuristic is chosen, as the expiration solely depends on the route so far.
    ///
    /// # Parameters
    /// - `first`: The first route stage to compare.
    /// - `second`: The second route stage to compare.
    ///
    /// # Returns
    /// - `Ordering::Greater` if `first` is considered greater than `second` based on the criteria.
    /// - `Ordering::Less` if `second` is considered greater than `first`.
    /// - `Ordering::Equal` if both stages are equal by all criteria.
    ///
    /// # Performance
    /// This function is marked with `#[inline(always)]` for potential performance optimizations.
    #[inline(always)]
    fn cmp(first: &RouteStage<NM, CM>, second: &RouteStage<NM, CM>) -> Ordering {
        if first.at_time_heuristic > second.at_time_heuristic {
            return Ordering::Greater;
        } else if first.at_time_heuristic < second.at_time_heuristic {
            return Ordering::Less;
        } else if first.hop_count_heuristic > second.hop_count_heuristic {
            return Ordering::Greater;
        } else if first.hop_count_heuristic < second.hop_count_heuristic {
            return Ordering::Less;
        } else if first.expiration < second.expiration {
            return Ordering::Greater;
        } else if first.expiration > second.expiration {
            return Ordering::Less;
        }
        Ordering::Equal
    }

    /// Checks if two `RouteStage` instances are equal based on specific criteria.
    ///
    /// Equality is determined by the following criteria:
    /// - `at_time_heuristic`: Both instances must have the same `at_time_heuristic`.
    /// - `hop_count_heuristic`: Both instances must have the same `hop_count_heuristic`.
    /// - `expiration`: Both instances must have the same `expiration`.
    ///
    /// # Parameters
    /// - `first`: The first route stage to check for equality.
    /// - `second`: The second route stage to check for equality.
    ///
    /// # Returns
    /// - `true` if `first` and `second` meet the criteria for equality.
    /// - `false` otherwise.
    ///
    /// # Performance
    /// This function is marked with `#[inline(always)]` for potential performance optimizations.
    #[inline(always)]
    fn eq(first: &RouteStage<NM, CM>, second: &RouteStage<NM, CM>) -> bool {
        first.at_time_heuristic == second.at_time_heuristic
            && first.hop_count_heuristic == second.hop_count_heuristic
            && first.expiration == second.expiration
    }
}

impl<NM: NodeManager, CM: ContactManager> HybridParentingOrd<NM, CM> for HeuristicSabr {
    // For SABR, the secondary metric to consider is the hop count.
    fn can_retain(prop: &RouteStage<NM, CM>, known: &RouteStage<NM, CM>) -> bool {
        prop.hop_count_heuristic < known.hop_count_heuristic
    }
    // Ignore expiration constraints to prioritize performance.
    fn must_prune(prop: &RouteStage<NM, CM>, known: &RouteStage<NM, CM>) -> bool {
        prop.at_time_heuristic <= known.at_time_heuristic && prop.hop_count_heuristic <= known.hop_count_heuristic
    }
}