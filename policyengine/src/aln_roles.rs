#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Mentor,
    Teacher,
    Learner,
    Operator,
    Regulator,
    System,
    Host,
    OrganicCpuOwner,
    SovereignKernel,
    NeuromorphSovereign, // NEUROMORPH-GOD alias (symbolic)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleSet {
    pub roles: Vec<Role>,
    /// Number of independent Regulator signatures observed for this decision.
    pub regulator_quorum: u8,
}

impl RoleSet {
    pub fn has(&self, r: Role) -> bool {
        self.roles.contains(&r)
    }

    /// “Neuromorph-god” composite predicate:
    /// Host + OrganicCPUOwner + SovereignKernel + regulator quorum.
    pub fn neuromorph_god_satisfied(&self, required_reg_quorum: u8) -> bool {
        self.has(Role::Host)
            && self.has(Role::OrganicCpuOwner)
            && self.has(Role::SovereignKernel)
            && self.regulator_quorum >= required_reg_quorum
    }
}

/// Kernel helper for ReversalConditions.
pub fn can_revert_capability(
    roles: &RoleSet,
    required_reg_quorum: u8,
    explicit_reversal_order: bool,
    no_safer_alternative: bool,
) -> bool {
    roles.neuromorph_god_satisfied(required_reg_quorum)
        && explicit_reversal_order
        && no_safer_alternative
}
