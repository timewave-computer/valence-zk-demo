# ZK cross-chain messaging application
We currently have two mailbox contracts deployed on Sepolia (Ethereum) and Pion-1 (Neutron).

| Sepolia | Pion-1 |
|---|---|
| 0x9151c571b53627Bc20Ce59F18B156AE8AFaADe7d | neutron1h967w282lz9tv8qxw5ch7a3cefndp4umexhpz9ehqa2d87gg80hsswsrlr |

Both contracts have the following storage layout:

| Chain | Slot | Data |
|---|---|---|
| Sepolia | 0 | Mapping(Uint256->string) |
| Sepolia | 1 | Uint256 |
| Pion-1 | 0 | Mapping(Uint128->string) |
| Pion-1 | 1 | Uint128 |

Where the mapping at slot `0` represents messages and the value at slot `1` represents the total amount of messages that have been sent.

| Chain | Default Account |
|---|---|
| Sepolia | 0x51df57D545074bA4b2B04b5f973Efc008A2fde6E |
| Pion-1 | neutron148w9spa5f9hcwgdy8cnejfel8ly6c2kdazuu94ja5dmy6zyet2ks6c49fd |
