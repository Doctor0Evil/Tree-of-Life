#pragma once
#include <string>
#include <stdexcept>
#include <algorithm>

namespace EcoNet {

struct SensorSample {
    double concentration;   // C(t), canonical units (e.g. ng/L, mg/L)
    double flow;            // Q(t), m^3/s
    double timestamp;       // seconds since epoch
};

struct JurisdictionRefs {
    double Cref_EPA;   // e.g. MCL or health advisory
    double Cref_EU;    // e.g. DWD / EQS
    double Cref_WHO;   // e.g. guideline value
    double IR;         // ingestion rate, L/day
    double BW;         // body weight, kg
};

inline double computeSupremeConcentration(const JurisdictionRefs& jr,
                                          double flow_m3s)
{
    // Collect admissible limits (concentration or dose-based surrogate)
    double Q = std::max(flow_m3s, 0.0);
    double IR_Lps = jr.IR / 86400.0; // L/s

    double dose_WHO = 0.0;
    if (jr.Cref_WHO > 0.0 && jr.BW > 0.0 && IR_Lps > 0.0) {
        // C = D * BW / IR, rearranged from dose model D = C * IR / BW
        dose_WHO = jr.Cref_WHO * jr.BW / IR_Lps;
    }

    double limits[3];
    int n = 0;
    if (jr.Cref_EPA > 0.0)      limits[n++] = jr.Cref_EPA;
    if (jr.Cref_EU > 0.0 && Q > 0.0) limits[n++] = jr.Cref_EU;
    if (dose_WHO > 0.0)         limits[n++] = dose_WHO;

    if (n == 0) {
        throw std::runtime_error("No admissible jurisdictional limits.");
    }
    return *std::min_element(limits, limits + n); // strictest constraint
}

struct CeimNodeState {
    std::string nodeId;
    std::string contaminantId;

    double hazardWeight;   // ω_x in [0, +∞), e.g. 3.0 for E. coli, 1.0 PFBS
    JurisdictionRefs refs;

    double volume;         // m^3, effective control volume (for kinetics)
    double k;              // 1/s, optional first-order kinetic parameter

    double Kn;             // accumulated impact (dimensionless)
    double t_last;         // last timestamp
    bool   has_last;

    CeimNodeState()
        : hazardWeight(1.0), volume(0.0), k(0.0),
          Kn(0.0), t_last(0.0), has_last(false) {}
};

inline double updateCeimNode(CeimNodeState& st, const SensorSample& s)
{
    if (!st.has_last) {
        st.t_last  = s.timestamp;
        st.has_last = true;
        return st.Kn;
    }

    double dt = s.timestamp - st.t_last;
    if (dt <= 0.0 || s.flow <= 0.0) {
        st.t_last = s.timestamp;
        return st.Kn;
    }

    // Optional kinetic closure for C_out (first-order in a CSTR view)
    double tau = st.volume > 0.0 ? st.volume / s.flow : 0.0;
    double Cout_model = (st.k > 0.0)
        ? s.concentration * std::exp(-st.k * tau)
        : s.concentration; // no removal modeled at kernel level

    // Supreme jurisdictional reference
    double Csup = computeSupremeConcentration(st.refs, s.flow);

    // Do not allow reported C_out to exceed Csup
    double Cout = std::min(Cout_model, Csup);

    double deltaC = s.concentration - Cout;
    if (deltaC <= 0.0) {
        st.t_last = s.timestamp;
        return st.Kn;
    }

    // Discrete CEIM increment: ω_x * ((Cin - Cout)/Csup) * Q * dt
    double increment = st.hazardWeight * (deltaC / Csup) * s.flow * dt;
    st.Kn += increment;

    st.t_last = s.timestamp;
    return st.Kn;
}

} // namespace EcoNet
