#![feature(let_chains, is_none_or)]

use std::rc::Rc;

use builder::{Definition, ProgramBuilder, Vars};
use TSPL::Parser;

pub mod builder;
pub mod checker;
pub mod compiler;
pub mod run;
pub mod syntax;

use run::{InteractionSystem, Net as RtNet, Tree as RtTree};
use syntax::{Item, Tree};

impl<'i> ProgramBuilder<'i> {
    pub fn tree_ast_to_rt(&mut self, tree: Tree) -> RtTree {
        match tree {
            Tree::Agent(name, aux) => RtTree::Agent {
                id: self.get_agent_id(name),
                aux: aux.into_iter().map(|x| self.tree_ast_to_rt(x)).collect(),
            },
            Tree::Var(id) => RtTree::Var {
                id: self.get_var_id(id),
            },
        }
    }
    fn build(&mut self) -> Result<(), String> {
        self.get_agent_id("::".to_string());
        self.get_agent_id(":".to_string());
        self.skip_trivia();
        while self.peek_one().is_some() {
            let mut idx = self.index;
            while let Ok(m) = self.parse_macro() {
                idx = m;
            }
            self.index = idx;
            self.skip_trivia();
            if !self.peek_one().is_some() {
                break;
            }
            match self.parse_item()? {
                Item::Definition(tree_left, tree_right, redexes) => {
                    let Tree::Agent(l_name, l_children) = tree_left else {
                        return Err("Invalid item found!".to_string());
                    };
                    let Tree::Agent(r_name, r_children) = tree_right else {
                        return Err("Invalid item found!".to_string());
                    };
                    let def = Definition {
                        left_id: self.get_agent_id(l_name),
                        left_children: vec![],
                        right_id: self.get_agent_id(r_name),
                        right_children: vec![],
                    };
                    self.vars = Vars {
                        net: RtNet::default(),
                        var_scope: Default::default(),
                    };
                    self.def.push(def);

                    for i in redexes {
                        let interaction = (
                            self.tree_ast_to_rt(i.left_tree),
                            self.tree_ast_to_rt(i.right_tree),
                        );
                        self.vars.net.interactions.push(interaction);
                    }

                    self.vars.net.normal();
                    for i in l_children {
                        let i = self.tree_ast_to_rt(i);
                        let i = self.vars.net.substitute(i);
                        self.def.last_mut().unwrap().left_children.push(i);
                    }
                    for i in r_children {
                        let i = self.tree_ast_to_rt(i);
                        let i = self.vars.net.substitute(i);
                        self.def.last_mut().unwrap().right_children.push(i);
                    }
                }
            }
        }
        Ok(())
    }
    fn build_interaction_system(&mut self) -> Rc<InteractionSystem> {
        let mut system = InteractionSystem::default();
        for definition in &self.def {
            let rule = run::InteractionRule {
                left_ports: definition.left_children.clone(),
                right_ports: definition.right_children.clone(),
            };
            system.add_rule(definition.left_id, definition.right_id, rule)
        }
        let system = Rc::new(system);
        self.interaction_system = Some(system.clone());
        system
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
    p.check_well_typedness();
}
