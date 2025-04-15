# ZK Rate calculation for a Cross Chain Vault
We currently have two mock vault contracts deployed on Sepolia (Ethereum) and Pion-1 (Neutron).

| Sepolia | Pion-1 |
|---|---|
| 0x8Fbd2549Dc447d229813ef5139b1aee8a9012eb3 | neutron148w9spa5f9hcwgdy8cnejfel8ly6c2kdazuu94ja5dmy6zyet2ks6c49fd |

Both contracts have the following storage layout:

| Chain | Slot | Data |
|---|---|---|
| Sepolia | 0 | Mapping(Address->Uint256) |
| Sepolia | 1 | Uint256 |
| Pion-1 | 0 | Mapping(Address->Uint128) |
| Pion-1 | 1 | Uint128 |

Where the mapping at slot `0` represents deposit balances and the value at slot `1` represents the total amount of LP shares that have been minted.
Since this is a vault, we have a default account in each mapping that we are interested in.

| Chain | Default Account |
|---|---|
| Sepolia | 0x51df57D545074bA4b2B04b5f973Efc008A2fde6E |
| Pion-1 | neutron148w9spa5f9hcwgdy8cnejfel8ly6c2kdazuu94ja5dmy6zyet2ks6c49fd |

The balance of each account on the respective chain has been initialized to `10` and the shares have also been initialized to `10`. Therefore our total cross-chain rate for this example is `10+10/10+10=20/20= 1 `.

The values in the respective contracts can be updated by anyone. You can review and deploy the contracts yourself, they are located in `examples/contracts/CHAIN_NAME-vault-contract`.