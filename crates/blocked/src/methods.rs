use std::collections::HashSet;

/// The list of anvil methods.
pub const ANVIL_METHODS: &[&str] = &[
    "anvil_autoImpersonateAccount",
    "anvil_dropTransaction",
    "anvil_dumpState",
    "anvil_enableTraces",
    "anvil_getAutomine",
    "anvil_impersonateAccount",
    "anvil_increaseTime",
    "anvil_loadState",
    "anvil_metadata",
    "anvil_mine",
    "anvil_mine_detailed",
    "anvil_nodeInfo",
    "anvil_removeBlockTimestampInterval",
    "anvil_reset",
    "anvil_revert",
    "anvil_setAutomine",
    "anvil_setBalance",
    "anvil_setBlockGasLimit",
    "anvil_setBlockTimestampInterval",
    "anvil_setChainId",
    "anvil_setCode",
    "anvil_setCoinbase",
    "anvil_setIntervalMining",
    "anvil_setLoggingEnabled",
    "anvil_setMinGasPrice",
    "anvil_setNextBlockBaseFeePerGas",
    "anvil_setNextBlockTimestamp",
    "anvil_setNonce",
    "anvil_setRpcUrl",
    "anvil_setStorageAt",
    "anvil_setTime",
    "anvil_snapshot",
    "anvil_stopImpersonatingAccount",
];

/// Returns a [`HashSet`] of blocked anvil methods.
pub fn blocked_method_set() -> HashSet<String> {
    ANVIL_METHODS
        .iter()
        .map(|method| method.to_ascii_lowercase())
        .collect()
}
