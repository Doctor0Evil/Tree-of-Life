# Tree-of-Life View Layer (`TreeofLife.rs`)

## Purpose

Tree-of-Life is a non-actuating, observer-only view layer for NewRow-Print! and NeuroPC. It maps existing neuromorphic safety signals (BiophysicalEnvelope snapshots, RoH, capability states) into 14 intuitive biophysical-logic assets:

- BLOOD, OXYGEN, WAVE, H2O, TIME, DECAY, LIFEFORCE, BRAIN, SMART, EVOLVE, POWER, TECH, FEAR, PAIN, NANO.

The goal is to provide an educational and fairness-oriented explanation surface for AI-chat, HUDs, and audits without adding any new control paths over capability, consent, or hardware.

Tree-of-Life MUST never:

- Change `CapabilityState` or consent states.
- Trigger or block stimulation, neuromodulation, or hardware operations.
- Modify `.stake.aln`, `.rohmodel.aln`, `.donutloop.aln`, or `.evolve.jsonl` beyond writing logs.

It MAY:

- Read envelope snapshots, RoH scores, and capability states.
- Compute normalized asset scores in `[0.0, 1.0]`.
- Emit advisory diagnostics and labels for humans and AI-chat.

## Files

- Rust module: `src/tree_of_life/TreeofLife.rs`
- Spec / docs: `docs/tree_of_life/TreeofLife.md`

## Inputs

Tree-of-Life reads a single neuromorphic snapshot via `TreeOfLifeInput`:

- `capability_state: CapabilityState`
  - Existing capability lattice: `CapModelOnly`, `CapLabBench`, `CapControlledHuman`, `CapGeneralUse`.
- `roh_score: RoHScore`
  - Risk-of-Harm scalar with RoH ≤ 0.3 in human-coupled states.
- `envelope: BiophysicalEnvelopeSnapshot`
  - Derived from `BiophysicalEnvelopeSpec` and shards (e.g., cognitive load, sleep arousal).
  - Must expose normalized features and WARN/RISK fractions, such as:
    - `hr_bpm_normalized`, `hrv_rmssd_normalized`
    - `eeg_alpha_power_norm`, `eeg_beta_power_norm`, `eeg_gamma_power_norm`, `eeg_alpha_cve_norm`
    - `info_axis_fraction`, `warn_axis_fraction`, `risk_axis_fraction`
    - `eda_warn_fraction`, `eda_risk_fraction`, `hr_warn_fraction`, `hr_risk_fraction`
    - `motion_warn_fraction`, `motion_risk_fraction`
    - `active_axis_count`
- `evolve_index: Option<u64>`
  - Optional evolution counter from `.evolve.jsonl` or equivalent.
- `epoch_index: Option<u64>`
  - Optional logical time index (epoch counter).

All of these come from existing, non-hypothetical Rust modules and ALN shards (BiophysicalEnvelopeSpec, RoH model, capability lattice).

## Outputs

### TreeOfLifeView

`TreeOfLife::from_snapshot(input: &TreeOfLifeInput) -> TreeOfLifeView`

Fields (all `f32`, normalized to `[0.0, 1.0]`):

- `blood`
  - Derived from normalized heart rate envelope (`hr_bpm_normalized`).
- `oxygen`
  - Derived from normalized HRV RMSSD (`hrv_rmssd_normalized`) as a proxy for autonomic reserve.
- `wave`
  - Aggregated from EEG alpha/beta/gamma power + alpha-envelope CVE (`eeg_alpha_power_norm`, etc.).
- `h2o`
  - Reserved for hydration/metabolic axes. Until those exist, fixed at `0.5` to remain neutral.
- `time`
  - Function of `epoch_index`, saturating in `[0.0, 1.0]` across a configurable epoch horizon.
- `decay`
  - Function of RoH, mapping `roh_score.value` in `[0.0, 0.3]` to `[0.0, 1.0]`.
- `lifeforce`
  - Defined as `1.0 - (RoH / 0.3)` and clamped to `[0.0, 1.0]`.
- `brain`
  - Encodes capability tier:
    - `CapModelOnly` → `0.25`
    - `CapLabBench` → `0.5`
    - `CapControlledHuman` → `0.75`
    - `CapGeneralUse` → `1.0`
- `smart`
  - Blend of `brain` and normalized `evolve_index` to indicate stable, evidence-backed evolution.
- `evolve`
  - Normalized `evolve_index` (e.g., log of safe evolution steps), truncated at `1.0`.
- `power`
  - Combines WARN/RISK fractions across envelope axes:
    - Higher WARN/RISK → higher POWER, representing intensity, not authority.
- `tech`
  - Combines capability tier and active-axis count as a complexity score.
- `fear`
  - Weighted combination of EDA and HR WARN/RISK fractions.
- `pain`
  - Combination of FEAR and motion WARN/RISK fractions.
- `nano`
  - Normalized count of discrete evolution events (e.g., from `.evolve.jsonl`).

All mappings are clamped via `clamp01` to prevent out-of-range values.

### TreeOfLifeDiagnostics

`TreeOfLife::diagnostics(view: &TreeOfLifeView) -> TreeOfLifeDiagnostics`

- `labels: Vec<String>`
  - Example labels:
    - `"balanced"` when `lifeforce` is high and `fear`/`pain` are low.
    - `"overloaded"` when `fear` or `pain` are high.
    - `"cooldown-recommended"` when `decay` is high.
- `cooldown_advised: bool`
  - `true` if FEAR/PAIN or DECAY exceed fixed thresholds.
  - This is a suggestion only; capability transitions must still be decided by the policy engine and OwnerDecision.
- `fairness_imbalance: bool`
  - Default `false` at single-snapshot level.
  - Intended to be set by higher-level logic comparing multiple subjects/roles.

## Invariants and Governance

1. **Non-actuation**
   - `TreeofLife.rs` MUST NOT:
     - Call hardware drivers.
     - Modify `CapabilityState`, consent tokens, or ReversalConditions.
     - Write to or alter `.stake.aln`, `.rohmodel.aln`, `.donutloop.aln`, `.evolve.jsonl` except through standard logging of its own outputs.

2. **Read-only over existing safety stack**
   - Tree-of-Life is a view over:
     - BiophysicalEnvelopeSpec and shards (cognitive load, sleep arousal, etc.).
     - Capability lattice (CapModelOnly → CapGeneralUse).
     - RoH model (≤ 0.3 in human-coupled states).
   - It introduces no new safety bounds and cannot weaken existing envelopes.

3. **Provenance and non-fiction**
   - Any numeric mapping (e.g., RoH scaling, capability mapping to BRAIN) must be:
     - Explicitly documented here.
     - Implemented exactly in `TreeofLife.rs`.
   - If future adjustments are made (e.g., new axes for H2O or NANO), they must reference:
     - An ALN shard with documented derivation, or
     - A dataset + procedure already accepted in the NewRow-Print!/NeuroPC governance stack.

4. **Fairness and “no overpowering”**
   - Tree-of-Life can be extended with higher-level functions that compare multiple `TreeOfLifeView` instances across roles/subjects and compute fairness metrics.
   - These fairness metrics are strictly advisory:
     - They may be logged and displayed.
     - They may not grant, revoke, or reweight access rights or capability tiers without going through existing CapabilityGuard / OwnerDecision / POLICYSTACK logic.

## Example Snapshot (JSON)

```json
{
  "capability_state": "CapControlledHuman",
  "roh_score": { "value": 0.12 },
  "tree_of_life_view": {
    "blood": 0.42,
    "oxygen": 0.68,
    "wave": 0.55,
    "h2o": 0.50,
    "time": 0.31,
    "decay": 0.40,
    "lifeforce": 0.60,
    "brain": 0.75,
    "smart": 0.71,
    "evolve": 0.32,
    "power": 0.28,
    "tech": 0.63,
    "fear": 0.22,
    "pain": 0.18,
    "nano": 0.10
  },
  "diagnostics": {
    "labels": ["balanced"],
    "cooldown_advised": false,
    "fairness_imbalance": false
  }
}
