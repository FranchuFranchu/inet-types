#![feature(let_chains, is_none_or)]

use std::{collections::BTreeMap, rc::Rc};

use builder::{Definition, ProgramBuilder, Vars};
use TSPL::Parser;

pub mod builder;
pub mod checker;
pub mod compiler;
pub mod run;
pub mod syntax;

use run::{InteractionSystem, Tree as RtTree, VarId};
use syntax::Tree;

impl<'i> ProgramBuilder<'i> {
    pub fn tree_ast_to_rt(&mut self, tree: Tree) -> RtTree {
        match tree {
            Tree::Agent(name, aux) => {
                let id = self.get_or_new_agent_id(name);
                self.set_arity(id, aux.len() as u64);
                RtTree::Agent {
                    id: id,
                    aux: aux.into_iter().map(|x| self.tree_ast_to_rt(x)).collect(),
                }
            }
            Tree::Var(id) => RtTree::Var {
                id: self.get_or_new_var_id(id),
            },
        }
    }
    fn peek_macro(&mut self) -> Result<bool, String> {
        let index = self.index;
        let r = if self.parse_ctr_name().is_ok() {
            self.skip_trivia();
            self.peek_one() == Some('[')
        } else {
            false
        };
        self.index = index;
        Ok(r)
    }
    fn enter(&mut self) {
        self.levels.push(builder::Level {
            agent_scope: BTreeMap::new(),
            vars: Vars {
                net: Default::default(),
                var_scope: BTreeMap::new(),
            },
        });
    }
    fn exit(&mut self) -> Option<builder::Level> {
        self.levels.pop()
    }
    fn maybe_parse_scope(&mut self) -> Result<(), String> {
        self.skip_trivia();
        if self.peek_one() == Some('{') {
            self.consume("{")?;
            self.parse_scope()?;
            self.consume("}")?;
            Ok(())
        } else {
            Ok(())
        }
    }
    fn parse_scope(&mut self) -> Result<(), String> {
        self.skip_trivia();
        while self.peek_one().is_some_and(|x| x != '}') {
            self.skip_trivia();
            if self.peek_macro()? {
                // Parse a macro
                self.parse_macro()?;
            } else {
                // Parse a tree
                let left = self.parse_tree()?;
                self.skip_trivia();
                match self.peek_one() {
                    Some('=') => {
                        self.consume("=")?;
                        let right = self.parse_tree()?;

                        let interaction = (self.tree_ast_to_rt(left), self.tree_ast_to_rt(right));
                        self.levels
                            .last_mut()
                            .unwrap()
                            .vars
                            .net
                            .interactions
                            .push(interaction);
                    }
                    Some('~') => {
                        self.consume("~")?;
                        let right = self.parse_tree()?;

                        self.enter();

                        let Tree::Agent(l_name, l_children) = left else {
                            return Err("Invalid item found!".to_string());
                        };
                        let Tree::Agent(r_name, r_children) = right else {
                            return Err("Invalid item found!".to_string());
                        };
                        let l_children: Vec<_> = l_children
                            .into_iter()
                            .map(|x| self.tree_ast_to_rt(x))
                            .collect();
                        let r_children: Vec<_> = r_children
                            .into_iter()
                            .map(|x| self.tree_ast_to_rt(x))
                            .collect();
                        self.maybe_parse_scope()?;
                        let mut scope = self.exit().unwrap();
                        self.levels
                            .last_mut()
                            .unwrap()
                            .agent_scope
                            .extend(scope.agent_scope);

                        scope.vars.net.normal();
                        let l_children = l_children
                            .into_iter()
                            .map(|x| scope.vars.net.substitute(x))
                            .collect();
                        let r_children = r_children
                            .into_iter()
                            .map(|x| scope.vars.net.substitute(x))
                            .collect();

                        let def = Definition {
                            left_id: self.get_or_new_agent_id(l_name),
                            left_children: l_children,
                            right_id: self.get_or_new_agent_id(r_name),
                            right_children: r_children,
                        };
                        self.def.push(def);
                    }
                    _ => {
                        // :(
                        todo!();
                    }
                }
            }
            self.skip_trivia();
        }
        Ok(())
    }
    fn build(&mut self) -> Result<(), String> {
        let a = self.get_or_new_agent_id(":".into());
        self.set_arity(a, 2);
        let a = self.get_or_new_agent_id("~".into());
        self.set_arity(a, 1);
        let a = self.get_or_new_agent_id("::".into());
        self.set_arity(a, 1);
        self.parse_scope()?;
        Ok(())
    }
    fn build_interaction_system(&mut self) -> Rc<InteractionSystem> {
        let antitype_agent = self.get_or_new_agent_id("~".to_string());
        let annotator_agent = self.get_or_new_agent_id("::".to_string());
        let arities = self.arities.clone();
        let def = self.def.clone();
        let _agent_scope_back = self.agent_scope_back.clone();
        let isys = Rc::new(InteractionSystem {
            get: Box::new(move |a, b| {
                for definition in &def {
                    //println!("{:?}", (agent_scope_back.get(&definition.left_id), agent_scope_back.get(&definition.right_id)));
                    //println!("{:?}", (agent_scope_back.get(&a), agent_scope_back.get(&b)));
                    //println!("\n    {:?}\n    {:?}\n    {:?}", a, b, definition);
                    if *a == definition.left_id && *b == definition.right_id {
                        //println!("Yay!");
                        return Some(run::InteractionRule {
                            left_ports: definition.left_children.clone(),
                            right_ports: definition.right_children.clone(),
                        });
                    }
                    if *b == definition.left_id && *a == definition.right_id {
                        //println!("Yay!");
                        return Some(run::InteractionRule {
                            left_ports: definition.right_children.clone(),
                            right_ports: definition.left_children.clone(),
                        });
                    }
                }
                if *a == antitype_agent && *b == antitype_agent {
                    return Some(run::InteractionRule {
                        left_ports: vec![RtTree::Var { id: VarId(0) }],
                        right_ports: vec![RtTree::Var { id: VarId(0) }],
                    });
                }
                if *a == antitype_agent {
                    return Some(run::InteractionRule {
                        left_ports: vec![RtTree::Agent {
                            id: crate::run::AgentId(b.0, (b.1 + 1) % 2),
                            aux: (0..arities[b])
                                .map(|x| RtTree::Agent {
                                    id: antitype_agent,
                                    aux: vec![RtTree::Var { id: VarId(x) }],
                                })
                                .collect(),
                        }],
                        right_ports: (0..arities[b])
                            .map(|x| RtTree::Var { id: VarId(x) })
                            .collect(),
                    });
                }
                if a.0 == b.0 && a.1 == (b.1 + 1) % 2 {
                    // A ~ ~A
                    // Interaction with the inverse
                    return Some(run::InteractionRule {
                        left_ports: (0..arities[a])
                            .map(|x| RtTree::Var { id: VarId(x) })
                            .collect(),
                        right_ports: (0..arities[a])
                            .map(|x| RtTree::Var { id: VarId(x) })
                            .collect(),
                    });
                }
                /*
                if *a == annotator_agent {
                    return Some(run::InteractionRule {
                        left_ports: (0..arities[a])
                            .map(|x| RtTree::Agent {
                                id: annotator_agent,
                                aux: vec![RtTree::Var { id: VarId(x) }],
                            })
                            .collect(),
                        right_ports: vec![RtTree::Agent {
                            id: annotator_agent,
                            aux: (0..arities[a])
                                .map(|x| RtTree::Var { id: VarId(x) })
                                .collect(),
                        }],
                    });
                }*/
                return None;
            }),
        });
        self.interaction_system = Some(isys.clone());
        isys
    }
}

fn main() {
    let s = std::fs::read_to_string(std::env::args().skip(1).next().unwrap()).unwrap();
    let mut p = ProgramBuilder::new(&s);
    match p.build() {
        Ok(o) => o,
        Err(e) => println!("{}", e),
    };
    p.build_interaction_system();
    p.check_completeness();
    p.check_inverse();
    p.check_well_typedness();
}
