use types::beacon_state::BeaconState;
use types::eth_spec::{EthSpec};

/// Contains arbitrary state stored by the simulation
/// This struct need not adhere to any official Eth2 spec -- it will store internal
/// simulation state in whatever manner is most convenient.
/// However, some types that ARE listed in a spec (eg. BeaconState) will still be spec-compliant
/// even if they happen to be stored in Store.
pub struct Store<T>
where
    T: EthSpec,
{
    current_beacon_state: BeaconState<T>,
}