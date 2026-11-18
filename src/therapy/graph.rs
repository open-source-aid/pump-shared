use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;
use core::time::Duration;

use crate::{
    Basal, Bolus, Extended, InsulinAmount, Percent, PumpMode, TherapyMachine, TherapyState,
    Transition,
};

const REGIONS: [&str; 4] = ["mode", "basal", "bolus", "extended"];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Edge {
    pub region: &'static str,
    pub from: &'static str,
    pub transition: &'static str,
    pub to: &'static str,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Graph {
    pub edges: BTreeSet<Edge>,
}

impl Graph {
    pub fn standard() -> Self {
        Self::explore(|state| state)
    }

    pub fn explore<M: TherapyMachine>(make: impl Fn(TherapyState) -> M) -> Self {
        let mut edges = BTreeSet::new();
        for from in sample_states() {
            for transition in sample_transitions() {
                let mut machine = make(from);
                if machine.apply(transition).is_err() {
                    continue;
                }
                let to = *machine.therapy();
                let (before, after) = (labels(&from), labels(&to));
                for (i, moved) in changed(&from, &to).into_iter().enumerate() {
                    if moved {
                        edges.insert(Edge {
                            region: REGIONS[i],
                            from: before[i],
                            transition: transition.name(),
                            to: after[i],
                        });
                    }
                }
            }
        }
        Self { edges }
    }

    pub fn to_mermaid(&self) -> String {
        let mut out = String::from("stateDiagram-v2\n    state Therapy {\n");
        for (i, region) in REGIONS.iter().enumerate() {
            if i > 0 {
                out.push_str("        --\n");
            }
            let _ = writeln!(out, "        state {region} {{");
            for edge in self.edges.iter().filter(|e| e.region == *region) {
                let _ = writeln!(
                    out,
                    "            {region}_{} --> {region}_{} : {}",
                    edge.from, edge.to, edge.transition
                );
            }
            out.push_str("        }\n");
        }
        out.push_str("    }\n");
        out
    }

    pub fn to_dot(&self) -> String {
        let mut out = String::from("digraph therapy {\n    rankdir=LR;\n");
        for region in REGIONS {
            let _ = writeln!(
                out,
                "    subgraph cluster_{region} {{\n        label=\"{region}\";"
            );
            for edge in self.edges.iter().filter(|e| e.region == region) {
                let _ = writeln!(
                    out,
                    "        \"{region}.{}\" -> \"{region}.{}\" [label=\"{}\"];",
                    edge.from, edge.to, edge.transition
                );
            }
            out.push_str("    }\n");
        }
        out.push_str("}\n");
        out
    }
}

fn units(hundredths: u32) -> InsulinAmount {
    InsulinAmount::from_hundredths(hundredths)
}

fn sample_states() -> Vec<TherapyState> {
    let hour = Duration::from_secs(3600);
    let modes = [
        PumpMode::Unknown,
        PumpMode::Off,
        PumpMode::Suspended,
        PumpMode::Running,
    ];
    let basals = [
        Basal::Unknown,
        Basal::Scheduled,
        Basal::Temporary {
            rate: Percent::new(150),
            duration: hour,
        },
    ];
    let boluses = [
        Bolus::Unknown,
        Bolus::Idle,
        Bolus::Immediate {
            requested: units(100),
            delivered: units(0),
        },
    ];
    let extendeds = [
        Extended::Unknown,
        Extended::Idle,
        Extended::Running {
            amount: units(100),
            duration: hour,
        },
    ];
    let mut states = Vec::new();
    for mode in modes {
        for basal in basals {
            for bolus in boluses {
                for extended in extendeds {
                    states.push(TherapyState {
                        mode,
                        basal,
                        bolus,
                        extended,
                    });
                }
            }
        }
    }
    states
}

fn sample_transitions() -> [Transition; 11] {
    let hour = Duration::from_secs(3600);
    [
        Transition::Suspended,
        Transition::Resumed,
        Transition::TurnedOff,
        Transition::BolusStarted {
            requested: units(100),
        },
        Transition::BolusProgressed {
            delivered: units(50),
        },
        Transition::BolusEnded,
        Transition::TempBasalStarted {
            rate: Percent::new(150),
            duration: hour,
        },
        Transition::TempBasalEnded,
        Transition::ExtendedStarted {
            amount: units(100),
            duration: hour,
        },
        Transition::ExtendedEnded,
        Transition::LinkLost,
    ]
}

fn labels(s: &TherapyState) -> [&'static str; 4] {
    [
        mode_name(s.mode),
        basal_name(s.basal),
        bolus_name(s.bolus),
        extended_name(s.extended),
    ]
}

fn changed(a: &TherapyState, b: &TherapyState) -> [bool; 4] {
    [
        a.mode != b.mode,
        a.basal != b.basal,
        a.bolus != b.bolus,
        a.extended != b.extended,
    ]
}

fn mode_name(mode: PumpMode) -> &'static str {
    match mode {
        PumpMode::Unknown => "Unknown",
        PumpMode::Off => "Off",
        PumpMode::Suspended => "Suspended",
        PumpMode::Running => "Running",
    }
}

fn basal_name(basal: Basal) -> &'static str {
    match basal {
        Basal::Unknown => "Unknown",
        Basal::Scheduled => "Scheduled",
        Basal::Temporary { .. } => "Temporary",
    }
}

fn bolus_name(bolus: Bolus) -> &'static str {
    match bolus {
        Bolus::Unknown => "Unknown",
        Bolus::Idle => "Idle",
        Bolus::Immediate { .. } => "Immediate",
    }
}

fn extended_name(extended: Extended) -> &'static str {
    match extended {
        Extended::Unknown => "Unknown",
        Extended::Idle => "Idle",
        Extended::Running { .. } => "Running",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edge(
        region: &'static str,
        from: &'static str,
        transition: &'static str,
        to: &'static str,
    ) -> Edge {
        Edge {
            region,
            from,
            transition,
            to,
        }
    }

    #[test]
    fn standard_graph_has_the_main_edges() {
        let graph = Graph::standard();
        assert!(graph
            .edges
            .contains(&edge("bolus", "Idle", "BolusStarted", "Immediate")));
        assert!(graph
            .edges
            .contains(&edge("bolus", "Immediate", "BolusProgressed", "Immediate")));
        assert!(graph
            .edges
            .contains(&edge("mode", "Suspended", "Resumed", "Running")));
        assert!(graph
            .edges
            .contains(&edge("mode", "Running", "TurnedOff", "Off")));
        assert!(graph
            .edges
            .contains(&edge("basal", "Scheduled", "TempBasalStarted", "Temporary")));
        assert!(graph
            .edges
            .contains(&edge("extended", "Running", "LinkLost", "Unknown")));
    }

    #[test]
    fn exports_mermaid_and_dot() {
        let graph = Graph::standard();
        let mermaid = graph.to_mermaid();
        assert!(mermaid.starts_with("stateDiagram-v2"));
        assert!(mermaid.contains("bolus_Idle --> bolus_Immediate : BolusStarted"));
        let dot = graph.to_dot();
        assert!(dot.starts_with("digraph therapy"));
        assert!(dot.contains("\"bolus.Idle\" -> \"bolus.Immediate\" [label=\"BolusStarted\"]"));
    }

    struct NeverResumes(TherapyState);

    impl TherapyMachine for NeverResumes {
        fn therapy(&self) -> &TherapyState {
            &self.0
        }

        fn therapy_mut(&mut self) -> &mut TherapyState {
            &mut self.0
        }

        fn apply(&mut self, transition: Transition) -> Result<(), crate::TransitionError> {
            if transition == Transition::Resumed {
                return self.0.apply(Transition::TurnedOff);
            }
            self.0.apply(transition)
        }
    }

    #[test]
    fn graph_follows_an_overridden_machine() {
        assert_eq!(
            NeverResumes(TherapyState::default()).therapy_mut().mode,
            PumpMode::Unknown
        );
        let graph = Graph::explore(NeverResumes);
        assert!(graph
            .edges
            .contains(&edge("mode", "Suspended", "Resumed", "Off")));
        assert!(!graph
            .edges
            .contains(&edge("mode", "Suspended", "Resumed", "Running")));
    }

    #[test]
    fn exported_files_are_up_to_date() {
        let graph = Graph::standard();
        assert_eq!(graph.to_mermaid(), include_str!("../../.docs/therapy.mmd"));
        assert_eq!(graph.to_dot(), include_str!("../../.docs/therapy.dot"));
    }
}
