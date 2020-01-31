use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};
use ssz_derive::{Decode as DeriveDecode, Encode as DeriveEncode};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, DeriveDecode, DeriveEncode)]
pub struct ShardState {

}
