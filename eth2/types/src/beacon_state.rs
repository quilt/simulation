// Makes use of patterns and code found in https://github.com/sigp/lighthouse
use crate::eth_spec::EthSpec;
use crate::execution_environment::ExecutionEnvironment;
use crate::shard_state::ShardState;
use crate::slot_epoch_root::Slot;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode as DeriveDecode, Encode as DeriveEncode};
use quilt_sim_proof_of_concept_ssz_types::VariableList;
// Traits must be in scope in order to use items on the trait
use typenum::marker_traits::Unsigned;

/// The state of the `BeaconChain` at some slot.
/// Full spec is here: https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase0/beacon-chain.md#beaconstate
/// SSZ spec is here: https://github.com/ethereum/eth2.0-specs/blob/dev/ssz/simple-serialize.md

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, DeriveDecode, DeriveEncode)]
#[serde(bound = "T: EthSpec")]
pub struct BeaconState<T>
where
    T: EthSpec,
{
    // Versioning
    //    genesis_time: u64,
    slot: Slot,
    //    fork: Fork,

    // History
    //    latest_block_header: BeaconBlockHeader,
    //    block_roots: FixedVector<Root, T::SlotsPerHistoricalRoot>,
    //    state_roots: FixedVector<Root, T::SlotsPerHistoricalRoot>,
    //    historical_roots: VariableList<Root, T::HistoricalRootsLimit>,

    // Eth1
    //    eth1_data: Eth1Data,
    //    eth1_data_votes: VariableList<Eth1Data, T::ValidatorRegistryLimit>,
    //    eth1_deposit_index: u64,

    // Registry
    //    validators: VariableList<Validator, T::ValidatorRegistryLimit>,
    //    balances: VariableList<Gwei, T::ValidatorRegistryLimit>,

    // Randomness
    //    randao_mixes: FixedVector<Root, T::EpochsPerHistoricalVector>,

    // Slashings
    //    slashings: FixedVector<Gwei, T::EpochsPerSlashingsVector>,

    // Attestations
    //    previous_epoch_attestations: VariableList<PendingAttestation, T::MaxPendingAttestations>,
    //    current_epoch_attestations: VariableList<PendingAttestation, T::MaxPendingAttestations>,

    // Finality
    //    justification_bits: BitVector<T::JustificationBitsLength>,
    //    previous_justified_checkpoint: Checkpoint,
    //    current_justified_checkpoint: Checkpoint,
    //    finalized_checkpoint: Checkpoint,

    // Phase 1
    pub shard_states: VariableList<ShardState<T>, T::MaxShards>,
    //    online_countdown: VariableList<OnlineEpochs, T::ValidatorRegistryLimit>,
    //    current_light_committee: CompactCommittee,
    //    next_light_committee: CompactCommittee,

    // Custody game
    //    exposed_derived_secrets: FixedVector<VariableList<ValidatorIndex, T::MaxEarlyDerivedSecretRevealsPerEpoch>, T::EarlyDerivedSecretPenaltyMaxFutureEpochs>,

    // Unspecced fields
    pub execution_environments: VariableList<ExecutionEnvironment<T>, T::MaxExecutionEnvironments>,
}

impl<T: EthSpec> BeaconState<T> {
    pub fn new() -> Self {
        // shard_states should initialize shard for each shard
        let initial_shard_state: ShardState<T> = ShardState::new();
        let shard_states_vec = vec![initial_shard_state; T::MaxShards::to_usize()];
        let shard_states = VariableList::new(shard_states_vec).unwrap();
        Self {
            execution_environments: VariableList::empty(),
            shard_states,
            slot: Slot::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Can't use these traits unless they're imported
    use crate::eth_spec::MainnetEthSpec;
    use ssz::{Decode, Encode};

    #[test]
    fn can_encode_and_decode_ssz() {
        let original: BeaconState<MainnetEthSpec> = BeaconState::new();
        let serialized: Vec<u8> = original.as_ssz_bytes();
        let deserialized: BeaconState<MainnetEthSpec> =
            BeaconState::from_ssz_bytes(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }
}
