use std::collections::BTreeMap;

use crate::{
    builder::ProgramBuilder,
    run::{AgentId, Net, Tree, VarId},
};

impl<'i> ProgramBuilder<'i> {
    pub fn get_type_of(&self, agent: AgentId) -> Option<AgentId> {
        let mut net = Net::default();
        net.system = self.interaction_system.as_ref().unwrap().clone();
        let type_var = net.new_var();
        let value_var = net.new_var();
        net.interact(
            Tree::Agent {
                id: self.immut_get_agent_id("::".to_string()),
                aux: vec![Tree::Agent {
                    id: self.immut_get_agent_id(":".to_string()),
                    aux: vec![Tree::Var { id: value_var }, Tree::Var { id: type_var }],
                }],
            },
            Tree::Agent {
                id: agent,
                aux: vec![],
            },
        );
        net.normal();
        if let Tree::Agent {
            id: type_id,
            aux: _,
        } = net.substitute(Tree::Var { id: type_var })
        {
            return Some(type_id);
        } else {
            return None;
        }
    }
    pub fn check_completeness(&mut self) {
        let mut instances: BTreeMap<AgentId, Vec<AgentId>> = BTreeMap::new();
        for i in self.agent_scope.values() {
            if let Some(t) = self.get_type_of(*i) {
                instances.entry(t).or_default().push(*i);
            }
        }
        let empty = vec![];
        for definition in self.def.iter() {
            for a in instances.get(&definition.left_id).unwrap_or(&empty) {
                for b in instances.get(&definition.right_id).unwrap_or(&empty) {
                    if !self.interaction_system.as_ref().unwrap().has_rule(*a, *b) {
                        let agents: BTreeMap<AgentId, String> = self
                            .agent_scope
                            .iter()
                            .map(|(k, v)| (v.clone(), k.clone()))
                            .collect();
                        println!(
                            "Completeness check failed:\n\
                			\tInteraction {a_ty} = {b_ty} is defined\n\
                			\t{a_val}: {a_ty}\n\
                			\t{b_val}: {b_ty}\n\
                			\tbut interaction {a_val} = {b_val} isn't",
                            a_val = agents.get(&a).unwrap(),
                            b_val = agents.get(&b).unwrap(),
                            a_ty = agents.get(&definition.left_id).unwrap(),
                            b_ty = agents.get(&definition.right_id).unwrap(),
                        );
                    }
                }
            }
        }
    }
    pub fn check_well_typedness(&self) {
        for def in self.def.iter() {
            let annotator = self.immut_get_agent_id("::".to_string());
            let annotation = self.immut_get_agent_id(":".to_string());
            if def.left_id == annotator || def.right_id == annotator {
                continue;
            }
            if def.left_id == annotation || def.right_id == annotation {
                continue;
            }
            let make_tree = |id| Tree::Agent {
                id: annotator,
                aux: vec![Tree::Var { id: id }],
            };
            let mut net = Net::default();
            net.system = self.interaction_system.as_ref().unwrap().clone();
            let left_vars: Vec<_> = def.left_children.iter().map(|_| net.new_var()).collect();
            let right_vars: Vec<_> = def.right_children.iter().map(|_| net.new_var()).collect();
            let trees_1: Vec<Tree> = {
                let mut net = net.clone();
                net.interact(
                    Tree::Agent {
                        id: def.left_id,
                        aux: left_vars.clone().into_iter().map(make_tree).collect(),
                    },
                    Tree::Agent {
                        id: def.right_id,
                        aux: right_vars.clone().into_iter().map(make_tree).collect(),
                    },
                );
                net.normal();
                left_vars
                    .clone()
                    .into_iter()
                    .chain(right_vars.clone().into_iter())
                    .map(|x| net.substitute_ref(&Tree::Var { id: x }))
                    .collect()
            };
            let trees_2: Vec<Tree> = {
                let v = net.new_var();
                net.interact(
                    Tree::Agent {
                        id: def.left_id,
                        aux: left_vars.clone().into_iter().map(make_tree).collect(),
                    },
                    Tree::Agent {
                        id: annotator,
                        aux: vec![Tree::Var { id: v }],
                    },
                );
                net.interact(
                    Tree::Agent {
                        id: def.right_id,
                        aux: right_vars.clone().into_iter().map(make_tree).collect(),
                    },
                    Tree::Agent {
                        id: annotator,
                        aux: vec![Tree::Var { id: v }],
                    },
                );
                net.normal();
                left_vars
                    .clone()
                    .into_iter()
                    .chain(right_vars.clone().into_iter())
                    .map(|x| net.substitute_ref(&Tree::Var { id: x }))
                    .collect()
            };

            fn compare(left: &Tree, right: &Tree, equiv: &mut BTreeMap<VarId, VarId>) -> bool {
                match (left, right) {
                    (Tree::Agent { id: idl, aux: auxl }, Tree::Agent { id: idr, aux: auxr }) => {
                        idl == idr
                            && auxl
                                .iter()
                                .zip(auxr.iter())
                                .all(|(a, b)| compare(a, b, equiv))
                    }
                    (Tree::Var { id: idl }, Tree::Var { id: idr }) => {
                        equiv.insert(*idl, *idr).is_none_or(|x| x == *idr)
                    }
                    _ => false,
                }
            }
            let mut map = BTreeMap::new();
            let eq = trees_1
                .iter()
                .zip(trees_2.iter())
                .all(|(a, b)| compare(a, b, &mut map));
            if !eq {
                let agents: BTreeMap<AgentId, String> = self
                    .agent_scope
                    .iter()
                    .map(|(k, v)| (v.clone(), k.clone()))
                    .collect();
                println!(
                    "Rule {} = {} is not well typed!",
                    agents.get(&def.left_id).unwrap(),
                    agents.get(&def.right_id).unwrap()
                );
                let mut scope = BTreeMap::new();
                let show_agent = |a| agents.get(&a).cloned().unwrap_or("?".to_string());
                let mut visited = vec![];
                for (a, b) in trees_1.iter().zip(trees_2.iter()) {
                    println!(
                        "{}\t|\t{}\t|\t{}",
                        net.show_tree(&show_agent, &mut scope, &mut visited, a),
                        net.show_tree(&show_agent, &mut scope, &mut visited, b),
                        compare(a, b, &mut map)
                    );
                }
            }
        }
    }
}
