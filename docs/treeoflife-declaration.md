# Tree‑of‑Life Declaration

## Purpose

Tree‑of‑Life (TOL) is a **non‑actuating diagnostic layer** that projects governed neuromorphic state into a small, human‑readable biophysical vocabulary for explanation, education, and fairness analysis. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/7b0a9e46-7e66-4cda-8397-100d94615c25/neuroprint-how-can-this-be-rep-fBJKSM3.QxWtu70GEWC.Fw.md)

TOL reads existing kernels (CapabilityState, BiophysicalEnvelopeSnapshot, RoH, evolveindex) and emits normalized scalar traits and advisory labels that can be logged, visualized, or used as evidence, but never as direct control inputs. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/1eaea4a0-75e4-475e-9ee9-d693020040fb/what-tree-of-life-traits-can-b-zDTYG1tUSZW..S2PZSkP.Q.md)

## Scope and Inputs

Tree‑of‑Life operates only on:

- CapabilityState (four‑tier lattice: MODELONLY, LABBENCH, CONTROLLEDHUMAN, GENERALUSE). [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/1eaea4a0-75e4-475e-9ee9-d693020040fb/what-tree-of-life-traits-can-b-zDTYG1tUSZW..S2PZSkP.Q.md)
- BiophysicalEnvelopeSnapshot built from BiophysicalEnvelopeSpec axes (EEG, HR/HRV, EDA, respiration, gaze, motion) with RoH ceiling 0.30 for CapControlledHuman. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/0d964317-c2c3-400a-81f6-f923ea23fc71/if-necessary-sanitize-the-code-7jDmbRJlT3SnSttCB78ZQg.md)
- RoHProjection (rohbefore, rohafter, rohceiling) with RoHafter ≥ RoHbefore and RoHafter ≤ rohceiling. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/1eaea4a0-75e4-475e-9ee9-d693020040fb/what-tree-of-life-traits-can-b-zDTYG1tUSZW..S2PZSkP.Q.md)
- Optional indices (evolveindex, epochindex, ledger references) for logging and fairness computations. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/7b0a9e46-7e66-4cda-8397-100d94615c25/neuroprint-how-can-this-be-rep-fBJKSM3.QxWtu70GEWC.Fw.md)

Tree‑of‑Life defines a pure function:

- TreeOfLifeInput → TreeOfLifeView, TreeOfLifeDiagnostics (no side effects, no I/O, no hardware calls). [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/1eaea4a0-75e4-475e-9ee9-d693020040fb/what-tree-of-life-traits-can-b-zDTYG1tUSZW..S2PZSkP.Q.md)

## TREE Assets (Biophysical Vocabulary)

TreeOfLifeView is a fixed vector of 14–15 normalized traits, each in \([0,1]\) and each a pure projection from governed telemetry: [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/7b0a9e46-7e66-4cda-8397-100d94615c25/neuroprint-how-can-this-be-rep-fBJKSM3.QxWtu70GEWC.Fw.md)

- BLOOD – cardiovascular strain / resource expenditure from HR/HRV. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/7b0a9e46-7e66-4cda-8397-100d94615c25/neuroprint-how-can-this-be-rep-fBJKSM3.QxWtu70GEWC.Fw.md)
- OXYGEN – autonomic reserve / adaptability from HRV + respiration / SpO₂. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/7b0a9e46-7e66-4cda-8397-100d94615c25/neuroprint-how-can-this-be-rep-fBJKSM3.QxWtu70GEWC.Fw.md)
- WAVE – composite brain‑wave index from EEG bandpower and alpha‑CVE. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/7b0a9e46-7e66-4cda-8397-100d94615c25/neuroprint-how-can-this-be-rep-fBJKSM3.QxWtu70GEWC.Fw.md)
- TIME – session / epoch progression, normalized. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/7b0a9e46-7e66-4cda-8397-100d94615c25/neuroprint-how-can-this-be-rep-fBJKSM3.QxWtu70GEWC.Fw.md)
- DECAY – normalized RoH proximity to rohceiling (e.g. RoH / 0.3, clamped to 1.0). [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/1eaea4a0-75e4-475e-9ee9-d693020040fb/what-tree-of-life-traits-can-b-zDTYG1tUSZW..S2PZSkP.Q.md)
- LIFEFORCE – complement of DECAY (1 − DECAY) as remaining reserve. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/7b0a9e46-7e66-4cda-8397-100d94615c25/neuroprint-how-can-this-be-rep-fBJKSM3.QxWtu70GEWC.Fw.md)
- BRAIN, SMART, EVOLVE, NANO – capability/evolution views from CapabilityState + evolveindex. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/7b0a9e46-7e66-4cda-8397-100d94615c25/neuroprint-how-can-this-be-rep-fBJKSM3.QxWtu70GEWC.Fw.md)
- POWER, TECH – intensity/complexity views derived from envelope WARN/RISK coverage and tier. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/1eaea4a0-75e4-475e-9ee9-d693020040fb/what-tree-of-life-traits-can-b-zDTYG1tUSZW..S2PZSkP.Q.md)
- FEAR, PAIN – distress views from EDA and motion WARN/RISK fractions, optionally subjective pain logs. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/7b0a9e46-7e66-4cda-8397-100d94615c25/neuroprint-how-can-this-be-rep-fBJKSM3.QxWtu70GEWC.Fw.md)

These traits:

- Are **read‑only diagnostics**; they never open new signals or control channels. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/1eaea4a0-75e4-475e-9ee9-d693020040fb/what-tree-of-life-traits-can-b-zDTYG1tUSZW..S2PZSkP.Q.md)
- Are fully determined by governed inputs; no speculative physiology is introduced. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/7b0a9e46-7e66-4cda-8397-100d94615c25/neuroprint-how-can-this-be-rep-fBJKSM3.QxWtu70GEWC.Fw.md)
- Are serialized into `.evolve.jsonl` and `.donutloop.aln` as log‑only fields, not as policy inputs. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/1eaea4a0-75e4-475e-9ee9-d693020040fb/what-tree-of-life-traits-can-b-zDTYG1tUSZW..S2PZSkP.Q.md)

## Invariants

Tree‑of‑Life MUST satisfy all of the following invariants in all deployments: [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/0d964317-c2c3-400a-81f6-f923ea23fc71/if-necessary-sanitize-the-code-7jDmbRJlT3SnSttCB78ZQg.md)

1. **Purity and Non‑Actuation**  
   - All Tree‑of‑Life functions are pure: they do not mutate CapabilityState, ConsentState, BiophysicalEnvelopeSpec, RoH models, ReversalConditions, or policy files. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/1eaea4a0-75e4-475e-9ee9-d693020040fb/what-tree-of-life-traits-can-b-zDTYG1tUSZW..S2PZSkP.Q.md)
   - Tree‑of‑Life MUST NOT call hardware drivers, actuators, stimulation interfaces, or capability transition APIs. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/1eaea4a0-75e4-475e-9ee9-d693020040fb/what-tree-of-life-traits-can-b-zDTYG1tUSZW..S2PZSkP.Q.md)

2. **Read‑Only Projection**  
   - TreeOfLifeView assets are pure functions of (CapabilityState, BiophysicalEnvelopeSnapshot, RoHProjection, evolveindex) and bounded in \([0,1]\). [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/1eaea4a0-75e4-475e-9ee9-d693020040fb/what-tree-of-life-traits-can-b-zDTYG1tUSZW..S2PZSkP.Q.md)
   - DECAY is a monotone function of RoH; LIFEFORCE is its complement; both inherit the RoH 0.30 ceiling contract. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/0d964317-c2c3-400a-81f6-f923ea23fc71/if-necessary-sanitize-the-code-7jDmbRJlT3SnSttCB78ZQg.md)

3. **Log‑Only Outputs**  
   - TOL outputs may only appear in: `.evolve.jsonl` records, `.donutloop.aln` decision ledger, HUD/AI‑chat views, offline analytics. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/1eaea4a0-75e4-475e-9ee9-d693020040fb/what-tree-of-life-traits-can-b-zDTYG1tUSZW..S2PZSkP.Q.md)
   - TOL outputs MUST NOT appear as predicates or guards in CapabilityTransitionRequest, ReversalConditions, PolicyStack, or RoH kernels. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/1eaea4a0-75e4-475e-9ee9-d693020040fb/what-tree-of-life-traits-can-b-zDTYG1tUSZW..S2PZSkP.Q.md)

4. **Sovereignty Preservation**  
   - Capability changes (upgrades or downgrades) are governed exclusively by CapabilityTransitionRequest, PolicyStack, consent, envelopes, RoH, and ReversalConditions. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/0d964317-c2c3-400a-81f6-f923ea23fc71/if-necessary-sanitize-the-code-7jDmbRJlT3SnSttCB78ZQg.md)
   - Tree‑of‑Life may only provide evidence (e.g., DECAY≈1.0, low LIFEFORCE) to external functions such as computenosaferalternative; it cannot set those flags itself. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/1eaea4a0-75e4-475e-9ee9-d693020040fb/what-tree-of-life-traits-can-b-zDTYG1tUSZW..S2PZSkP.Q.md)

5. **Deviceless Doctrine**  
   - Tree‑of‑Life is Tier‑2 diagnostic only; it may support envelope tightening, cooldown suggestions, or pausing within a capability tier, but cannot change capability tiers or neurorights floors. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/0d964317-c2c3-400a-81f6-f923ea23fc71/if-necessary-sanitize-the-code-7jDmbRJlT3SnSttCB78ZQg.md)

## Allowed Uses

Under this declaration, Tree‑of‑Life may be used for:

- **Explanation and Education**  
  - Rendering current state in HUDs and AI‑chat (“BLOOD high, DECAY rising, LIFEFORCE low”) to support self‑awareness and teaching. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/7b0a9e46-7e66-4cda-8397-100d94615c25/neuroprint-how-can-this-be-rep-fBJKSM3.QxWtu70GEWC.Fw.md)

- **Diagnostics and Fairness**  
  - Generating human‑interpretable dashboards and reports (e.g., energy budgets, overload history, fairness panels) that aggregate TREE assets over time and across subjects, without feeding into capability kernels. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/7b0a9e46-7e66-4cda-8397-100d94615c25/neuroprint-how-can-this-be-rep-fBJKSM3.QxWtu70GEWC.Fw.md)

- **Evidence for Safety Functions**  
  - Supplying scalar evidence to separate, audited functions that compute advisory predicates (CALMSTABLE, OVERLOADED, UNFAIRDRAIN, RECOVERY, nosaferalternative), which themselves remain non‑actuating and log‑only. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/7b0a9e46-7e66-4cda-8397-100d94615c25/neuroprint-how-can-this-be-rep-fBJKSM3.QxWtu70GEWC.Fw.md)

- **Simulation and Research**  
  - Serving as the mid‑level state in simulation sandboxes that generate synthetic TreeOfLifeView trajectories under BiophysicalEnvelopeSpec and RoH 0.30, with outputs written to JSONL for analysis only. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/0d964317-c2c3-400a-81f6-f923ea23fc71/if-necessary-sanitize-the-code-7jDmbRJlT3SnSttCB78ZQg.md)

## neuroprint!
neuroprint! is a pure, non‑actuating projection layer. It MAY read CapabilityState, ConsentState, BiophysicalEnvelopeSnapshot, RoHProjection, and neurorights policy views, and MAY emit NeuroPrintView structures and advisory labels, but it MUST NOT mutate, propose changes to, or open any write‑path into CapabilityState, ConsentState, BiophysicalEnvelopeSpec, RoH models, ReversalConditions, PolicyStack, or any hardware driver. All neuroprint! outputs are logs or views only, serialized into .evolve.jsonl / .donutloop.aln and HUD surfaces.

## Forbidden Uses

Tree‑of‑Life outputs MUST NOT be used to:

- Directly grant, revoke, or downgrade capabilities or rights. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/0d964317-c2c3-400a-81f6-f923ea23fc71/if-necessary-sanitize-the-code-7jDmbRJlT3SnSttCB78ZQg.md)
- Bypass or weaken BiophysicalEnvelopeSpec, RoH ceilings, neurorights, or PolicyStack constraints. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/0d964317-c2c3-400a-81f6-f923ea23fc71/if-necessary-sanitize-the-code-7jDmbRJlT3SnSttCB78ZQg.md)
- Drive any actuation, stimulation, or motion control pathway. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/1eaea4a0-75e4-475e-9ee9-d693020040fb/what-tree-of-life-traits-can-b-zDTYG1tUSZW..S2PZSkP.Q.md)
- Enter CapabilityTransitionRequest, ReversalConditions, PolicyStack, or OwnerDecision as gating predicates or reward signals. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_6557a17d-eabd-4cdc-aa7b-3dbedb30e0c5/1eaea4a0-75e4-475e-9ee9-d693020040fb/what-tree-of-life-traits-can-b-zDTYG1tUSZW..S2PZSkP.Q.md)

Any violation of these rules is a specification error and MUST be treated as a critical safety defect in the deployment.
