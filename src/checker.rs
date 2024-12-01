use std::collections::{BTreeMap, BTreeSet};

use crate::{
    builder::ProgramBuilder,
    run::{AgentId, Net, Tree},
};

impl<'i> ProgramBuilder<'i> {
    pub fn can_connect_to_set(&self, set: BTreeSet<AgentId>) -> impl Iterator<Item = AgentId> + '_ {
        let system = self.interaction_system.as_ref().unwrap().clone();
        self.agent_list()
            .filter(move |x| set.iter().any(|y| system.has_rule(*x, *y)))
    }
    pub fn can_connect_to(&self, a: AgentId) -> impl Iterator<Item = AgentId> + '_ {
        let system = self.interaction_system.as_ref().unwrap().clone();
        self.agent_list().filter(move |x| system.has_rule(*x, a))
    }
    pub fn is_subtype_of(&self, sub: AgentId, sup: AgentId) -> bool {
        let system = self.interaction_system.as_ref().unwrap().clone();
        for a in self.agent_list() {
            if system.has_rule(a, sup) && !system.has_rule(a, sub) {
                return false;
            }
        }
        return true;
    }
    pub fn get_type_of(&self, agent: AgentId) -> Option<AgentId> {
        let mut net = Net::default();
        net.system = self.interaction_system.as_ref().unwrap().clone();
        let type_var = net.new_var();
        let value_var = net.new_var();
        net.interact(
            Tree::Agent {
                id: self.get_agent_id("::").unwrap(),
                aux: vec![Tree::Agent {
                    id: self.get_agent_id(":").unwrap(),
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
        for i in self.agent_list() {
            if let Some(t) = self.get_type_of(i) {
                instances.entry(t).or_default().push(i);
            }
        }
        let empty = vec![];
        for ta in self.agent_list() {
            for tb in self.agent_list() {
                if ta <= tb && self.interaction_system.as_ref().unwrap().has_rule(ta, tb) {
                    for a in instances.get(&ta).unwrap_or(&empty) {
                        for b in instances.get(&tb).unwrap_or(&empty) {
                            if a <= b && !self.interaction_system.as_ref().unwrap().has_rule(*a, *b)
                            {
                                println!(
                                    "Completeness check failed:\n\
		                			\tInteraction {a_ty} ~ {b_ty} is defined\n\
		                			\t{a_val}: {a_ty}\n\
		                			\t{b_val}: {b_ty}\n\
		                			\tbut interaction {a_val} ~ {b_val} isn't",
                                    a_val = self.agent_scope_back.get(&a).unwrap(),
                                    b_val = self.agent_scope_back.get(&b).unwrap(),
                                    a_ty = self.agent_scope_back.get(&ta).unwrap(),
                                    b_ty = self.agent_scope_back.get(&tb).unwrap(),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn check_inverse(&mut self) {
        let system = self.interaction_system.as_ref().unwrap().clone();
        for a in self.agent_list() {
            for b in self.can_connect_to(a) {
                let b = self.agent_inverse(b);
                for c in self.can_connect_to(b) {
                    if !system.has_rule(a, c) {
                        println!(
                            "Inverse check failed:\n\
	                			\tInteraction {a} ~ {b} is defined\n\
	                			\tInteraction {b_inv} ~ {c} is defined\n\
	                			\twhich means {c} <= {b}, {a} <= {b_inv}\n\
	                			\tbut interaction {a} ~ {c} isn't defined",
                            a = self.agent_scope_back.get(&a).unwrap(),
                            b = self.agent_scope_back.get(&self.agent_inverse(b)).unwrap(),
                            b_inv = self.agent_scope_back.get(&b).unwrap(),
                            c = self.agent_scope_back.get(&c).unwrap(),
                        );
                    }
                }
            }
        }
    }
    pub fn check_well_typedness(&self) {
        for def in self.def.iter() {
            let mut show = false;

            let annotation = self.get_agent_id(":").unwrap();
            let annotator = self.get_agent_id("::").unwrap();
            let antitype_agent = self.get_agent_id("~").unwrap();
            if def.left_id == annotation
                || def.right_id == annotation
                || def.left_id == annotator
                || def.right_id == annotator
            {
                continue;
            }
            let make_tree = |id| Tree::Var { id };
            let _make_ann = |v, t| Tree::Agent {
                id: annotation,
                aux: vec![v, t],
            };
            let make_annotator = |v| Tree::Agent {
                id: annotator,
                aux: vec![v],
            };
            let mut net = Net::default();
            net.system = self.interaction_system.as_ref().unwrap().clone();
            let left_vars: Vec<_> = def
                .left_children
                .iter()
                .map(|_| make_tree(net.new_var()))
                .collect();
            let right_vars: Vec<_> = def
                .right_children
                .iter()
                .map(|_| make_tree(net.new_var()))
                .collect();
            let v = make_tree(net.new_var());

            net.interact(
                Tree::Agent {
                    id: def.left_id,
                    aux: left_vars
                        .clone()
                        .into_iter()
                        .map(|a| make_annotator(a))
                        .collect(),
                },
                make_annotator(v.clone()),
            );
            net.interact(
                Tree::Agent {
                    id: def.right_id,
                    aux: right_vars
                        .clone()
                        .into_iter()
                        .map(|a| make_annotator(a))
                        .collect(),
                },
                make_annotator(v),
            );
            net.interact(
                Tree::Agent {
                    id: def.left_id,
                    aux: left_vars
                        .into_iter()
                        .map(|a| {
                            make_annotator(Tree::Agent {
                                id: antitype_agent,
                                aux: vec![a],
                            })
                        })
                        .collect(),
                },
                Tree::Agent {
                    id: def.right_id,
                    aux: right_vars
                        .into_iter()
                        .map(|a| {
                            make_annotator(Tree::Agent {
                                id: antitype_agent,
                                aux: vec![a],
                            })
                        })
                        .collect(),
                },
            );
            let original_net = net.clone();
            net.normal();

            if show {
                println!("Checking a reduction...");
                let mut scope = BTreeMap::new();
                let mut net = original_net.clone();
                println!(
                    "{}",
                    net.show_net(&|x| self.show_agent(x), &mut scope, false)
                );
                while let Some((a, b)) = net.interactions.pop() {
                    net.interact(a, b);
                    print!(
                        "----------------------------\n{}",
                        net.show_net(&|x| self.show_agent(x), &mut scope, false)
                    );
                }
            }

            if !net.stuck.is_empty() {
                println!(
                    "Rule {} = {} is not well typed!",
                    self.agent_scope_back.get(&def.left_id).unwrap(),
                    self.agent_scope_back.get(&def.right_id).unwrap()
                );
                let mut scope = BTreeMap::new();
                println!(
                    "Original net:\n{}",
                    original_net.show_net(&|x| self.show_agent(x), &mut scope, false)
                );
                println!(
                    "----------------------------\n{}",
                    net.show_net(&|x| self.show_agent(x), &mut scope, false)
                );
            }
        }
    }
}
