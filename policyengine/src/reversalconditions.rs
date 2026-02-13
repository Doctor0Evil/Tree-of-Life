use crate::alnroles::{RoleSet, canrevertcapability};
use crate::alncore::{
    CapabilityState,
    CapabilityTransitionRequest,
    Decision,
    DecisionReason,
    PolicyStack,
};
use crate::rohmodel::RoHScore;
use crate::envelope::EnvelopeContextView;
use crate::policy::reversal::ReversalPolicyFlags;

/// Pure, side-effect-free context for evaluating neuromorph evolution reversals.
/// This is the minimal state tuple the kernel needs, aligned with
/// SECTION,REVERSAL-POLICY, SECTION,ROLES / SECTION,ROLE-COMPOSITION,
/// BiophysicalEnvelopeSpec, and the RoH model.[file:21]
#[derive(Debug, Clone)]
pub struct ReversalContext<'a> {
    /// Original capability transition request (from → to, requester, consent, jurisdiction).
    pub base: &'a CapabilityTransitionRequest,

    /// Risk-of-Harm score before the proposed transition (actual state).
    pub roh_before: RoHScore,

    /// Risk-of-Harm score after the proposed transition (counterfactual / projected).
    pub roh_after: RoHScore,

    /// Shard-level reversal policy flags frozen from SECTION,REVERSAL-POLICY
    /// (allowneuromorphreversal, explicitreversalorder, nosaferalternative).[file:21]
    pub reversal_flags: ReversalPolicyFlags,

    /// Active role set, including Host, OrganicCpuOwner, Regulator, SovereignKernel, etc.,
    /// used to satisfy the neuromorphgodsatisfied / NeuromorphSovereign predicate.[file:21]
    pub roles: &'a RoleSet,

    /// Pre-collapsed PolicyStack summary
    /// (BASEMEDICAL ∧ BASEENGINEERING(if used) ∧ JURISLOCAL ∧ QUANTUMAISAFETY).[file:21]
    pub policy_stack: &'a PolicyStack,

    /// View over envelope outputs for this subject/session, including any
    /// envelope invariants needed to detect structural violations.[file:21]
    pub envelope_ctx: &'a EnvelopeContextView,
}

/// Top-level evaluation entrypoint.
///
/// This function is pure and total: it reads only from `ctx` and returns a `Decision`,
/// and has no IO, no global state, and no ledger writes.[file:21]
pub fn evaluate_reversal(ctx: &ReversalContext) -> Decision {
    // 1. Diagnostic isolation / non-actuation: if this request originates from a
    // purely diagnostic or HUD-only path, reject any attempt to change capability.[file:21]
    if ctx.envelope_ctx.diag_event() {
        return Decision::Denied(DecisionReason::DeniedDiagnosticOnlyStep);
    }

    // 2. Enforce RoH monotonicity and ceiling in CapControlledHuman:
    // roh_after ≥ roh_before and roh_after ≤ roh_ceiling (0.30).[file:21]
    if is_cap_controlled_human(ctx.base.to_state()) {
        if ctx.roh_after.value() < ctx.roh_before.value()
            || ctx.roh_after.value() > ctx.roh_after.ceiling()
        {
            return Decision::Denied(DecisionReason::DeniedRoHViolation);
        }
    }

    // 3. Classify transition: if this is not a neuromorph evolution downgrade,
    // delegate to the normal CapabilityTransitionRequest::evaluate path.[file:21]
    if !is_neuromorph_evolution_downgrade(ctx.base.from_state(), ctx.base.to_state()) {
        return CapabilityTransitionRequest::evaluate(ctx.base);
    }

    // From here on, we are in the last-resort neuromorph evolution downgrade path.

    // 4. Global downgrade enable flag (default deny).
    // If allowneuromorphreversal is false, no evolution downgrade is considered.[file:21]
    if !ctx.reversal_flags.allow_neuromorph_reversal {
        return Decision::Denied(DecisionReason::DeniedReversalNotAllowedInTier);
    }

    // 5. Sovereign quorum / NEUROMORPHGOD composite role:
    // Host ∧ OrganicCpuOwner ∧ SovereignKernel ∧ Regulator quorum≥N must hold.[file:21]
    if !canrevertcapability(
        ctx.roles,
        ctx.reversal_flags.explicit_reversal_order,
        ctx.reversal_flags.no_safer_alternative,
    ) {
        // If composite predicate fails, classify as illegal downgrade by non-regulator.[file:21]
        return Decision::Denied(DecisionReason::DeniedIllegalDowngradeByNonRegulator);
    }

    // 6. Explicit reversal order and no safer alternative gate.
    // Both flags are required; nosaferalternative is computed upstream by
    // compute_nosafer_alternative(...) over envelope / Tree-of-Life logs.[file:21]
    if !ctx.reversal_flags.explicit_reversal_order
        || !ctx.reversal_flags.no_safer_alternative
    {
        return Decision::Denied(DecisionReason::DeniedNoSaferAlternativeNotProved);
    }

    // 7. PolicyStack hard gate:
    // BASEMEDICAL ∧ BASEENGINEERING(if used) ∧ JURISLOCAL ∧ QUANTUMAISAFETY must all pass.[file:21]
    if !ctx.policy_stack.all_pass() {
        return Decision::Denied(DecisionReason::DeniedPolicyStackFailure(
            ctx.policy_stack.failed_shard_name().unwrap_or_else(|| "UNKNOWN".to_string()),
        ));
    }

    // 8. Envelope invariants:
    // Envelopes must not be violated (minsafemaxsafe bands, RoH-compatible projections). [file:21]
    if ctx.envelope_ctx.envelope_violation() {
        return Decision::Denied(DecisionReason::DeniedEnvelopeViolation);
    }

    // If we reach this point, all structural and governance gates have passed:
    // - Diagnostic isolation satisfied (no diag-only origin).
    // - RoH monotonicity and ceiling satisfied.
    // - Transition is a true neuromorph evolution downgrade.
    // - allowneuromorphreversal == true (Tier-1 flag).
    // - canrevertcapability == true (neuromorphgodsatisfied ∧ explicitreversalorder ∧ nosaferalternative).
    // - PolicyStack_all_pass == true.
    // - Envelope invariants satisfied.[file:21]
    //
    // By design, there is no heuristic weighting or trade-off between these gates.[file:21]
    Decision::Allowed
}

/// Helper: determine if a lattice transition is a neuromorph evolution downgrade.
/// This is defined as a move from CapControlledHuman or CapGeneralUse downwards
/// in the CapabilityState lattice.[file:21]
fn is_neuromorph_evolution_downgrade(from: CapabilityState, to: CapabilityState) -> bool {
    use CapabilityState::*;
    match (from, to) {
        (CapControlledHuman, CapLabBench)
        | (CapControlledHuman, CapModelOnly)
        | (CapGeneralUse, CapControlledHuman)
        | (CapGeneralUse, CapLabBench)
        | (CapGeneralUse, CapModelOnly) => true,
        _ => false,
    }
}

/// Helper: check if a state is CapControlledHuman, used for RoH ceiling logic.[file:21]
fn is_cap_controlled_human(state: CapabilityState) -> bool {
    matches!(state, CapabilityState::CapControlledHuman)
}
