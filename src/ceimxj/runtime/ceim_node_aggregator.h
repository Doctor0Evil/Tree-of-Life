struct NodeImpactState {
    std::string nodeId;
    std::string contaminantId;
    double hazardWeight;   // ω_x
    double Cepa, Leu, Dwho;
    double IR, BW;
    double k;              // kinetic parameter if used
    double volume;
    double Kn;             // accumulated impact
    double tlast;
    bool   haslast;
};

// src/ceimxj/runtime/ceim_node_aggregator.cpp
double updateNodeImpact(NodeImpactState& st, const SensorSample& s) {
    if (!st.haslast) {
        st.tlast = s.timestamp;
        st.haslast = true;
        return st.Kn;
    }
    double dt = s.timestamp - st.tlast;
    if (dt <= 0.0) {
        st.tlast = s.timestamp;
        return st.Kn;
    }

    double tau = st.volume / s.flow;
    double Cout_model = s.concentration * std::exp(-st.k * tau);

    double Csup = computeSupremeConcentration(
        st.Cepa, st.Leu, st.Dwho, s.flow, st.IR, st.BW
    );
    double Cout = std::min(Cout_model, Csup);

    double deltaC = s.concentration - Cout;
    if (deltaC <= 0.0) {
        st.tlast = s.timestamp;
        return st.Kn;
    }

    double increment = st.hazardWeight * (deltaC / Csup) * s.flow * dt;
    st.Kn += increment;
    st.tlast = s.timestamp;
    return st.Kn;
}
