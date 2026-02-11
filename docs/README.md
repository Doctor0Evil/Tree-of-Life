# MicroSociety Tree-of-Life Simulator (Non‑Actuating Sandbox)

This crate defines a bounded, non‑actuating 1D MicroSociety sandbox for educational and research use.[file:1]  
It models agents on a 1D lattice with normalized Tree‑style assets and an internal observer ledger. All outputs are advisory and log‑only.

## Safety and Scope

- Non‑actuating: no hardware control, no writes to external capability or consent systems.[file:1]
- Bounded: all scalar assets are clamped to \[0,1], random events are local and limited.[file:2]
- Ledger: append‑only, hash‑linked DeedEvent list for diagnostics only.[file:1]

## Example

```bash
cargo run -- simulate-cycles --cycles 10 --agent-count 20
cargo run -- compute-rights --agent-id agent_0
