/// Snapshot serde-safe AccountsLtHash
#[cfg_attr(feature = "frozen-abi", derive(AbiExample))]
#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SerdeAccountsLtHash(
    // serde only has array support up to 32 elements; anything larger needs to be handled manually
    // see https://github.com/serde-rs/serde/issues/1937 for more information
    #[serde_as(as = "[_; 1024]")] pub [u16; 1024],
);
