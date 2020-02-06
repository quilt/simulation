// Copied from file of the same name at https://github.com/sigp/lighthouse
// with some modifications and deletions. Specifically, removed code that wasn't necessary for this
// repository, updated to use explicit imports, and added additional values to the spec definition.
use crate::beacon_state::BeaconState;
use crate::slot_epoch_root::Epoch;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use typenum::{
    Unsigned, U0, U1, U1024, U1099511627776, U128, U16, U16777216, U2048, U32, U4, U4096, U64,
    U65536, U8, U8192,
};

pub trait EthSpec: 'static + Default + Sync + Send + Clone + Debug + PartialEq {
    /*
     * Unspecced values
     */
    type MaxExecutionEnvironments: Unsigned + Clone + Sync + Send + Debug + PartialEq;

    //    /*
    //     * Constants
    //     */
    //    type JustificationBitsLength: Unsigned + Clone + Sync + Send + Debug + PartialEq + Default;
    //    /*
    //     * Misc
    //     */
    //    type MaxValidatorsPerCommittee: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    //    /*
    //     * Initial values
    //     */
    //    type GenesisEpoch: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    //    /*
    //     * Time parameters
    //     */
    //    type SlotsPerEpoch: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    //    type SlotsPerEth1VotingPeriod: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    //    type SlotsPerHistoricalRoot: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    //    /*
    //     * State list lengths
    //     */
    //    type EpochsPerHistoricalVector: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    //    type EpochsPerSlashingsVector: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    //    type HistoricalRootsLimit: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    //    type ValidatorRegistryLimit: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    //    /*
    //     * Max operations per block
    //     */
    //    type MaxProposerSlashings: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    //    type MaxAttesterSlashings: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    //    type MaxAttestations: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    //    type MaxDeposits: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    //    type MaxVoluntaryExits: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    //    /*
    //     * Derived values (set these CAREFULLY)
    //     */
    //    /// The length of the `{previous,current}_epoch_attestations` lists.
    //    ///
    //    /// Must be set to `MaxAttestations * SlotsPerEpoch`
    //    // NOTE: we could safely instantiate this by using type-level arithmetic, but doing
    //    // so adds ~25s to the time required to type-check this crate
    //    type MaxPendingAttestations: Unsigned + Clone + Sync + Send + Debug + PartialEq;

    //    /// Must be set to `MaxEarlyDerivedSecretReveals * SlotsPerEpoch
    //    type MaxEarlyDerivedSecretRevealsPerEpoch: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    //    // Phase 1
    //    type EarlyDerivedSecretPenaltyMaxFutureEpochs: Unsigned + Clone + Sync + Send + Debug + PartialEq;

    type MaxShards: Unsigned + Clone + Sync + Send + Debug + PartialEq;
}

/// Ethereum Foundation specifications.
///
/// Spec v0.9.1
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct MainnetEthSpec;

impl EthSpec for MainnetEthSpec {
    type MaxExecutionEnvironments = U65536;
    //    type JustificationBitsLength = U4;
    //    type MaxValidatorsPerCommittee = U2048;
    //    type GenesisEpoch = U0;
    //    type SlotsPerEpoch = U32;
    //    type SlotsPerEth1VotingPeriod = U1024;
    //    type SlotsPerHistoricalRoot = U8192;
    //    type EpochsPerHistoricalVector = U65536;
    //    type EpochsPerSlashingsVector = U8192;
    //    type HistoricalRootsLimit = U16777216;
    //    type ValidatorRegistryLimit = U1099511627776;
    //    type MaxProposerSlashings = U16;
    //    type MaxAttesterSlashings = U1;
    //    type MaxAttestations = U128;
    //    type MaxDeposits = U16;
    //    type MaxVoluntaryExits = U16;
    //    type MaxPendingAttestations = U4096; // 128 max attestations * 32 slots per epoch
    type MaxShards = U1024;
}

pub type FoundationBeaconState = BeaconState<MainnetEthSpec>;
