use std::{collections::BTreeMap, rc::Rc};

use TSPL::Parser;

use crate::run::{AgentId, InteractionSystem, Net, Tree, VarId};

#[derive(Debug)]
pub struct Definition {
    pub left_id: AgentId,
    pub left_children: Vec<Tree>,
    pub right_id: AgentId,
    pub right_children: Vec<Tree>,
}

#[derive(Debug, Default)]
pub struct Vars {
    pub net: Net,
    pub var_scope: BTreeMap<String, VarId>,
}

#[derive(Debug)]
pub struct ProgramBuilder<'i> {
    pub input: &'i str,
    pub index: usize,
    pub def: Vec<Definition>,
    pub agent_scope: BTreeMap<String, AgentId>,
    pub next_agent_id: u64,
    pub interaction_system: Option<Rc<InteractionSystem>>,
    pub vars: Vars,
}
impl<'i> ProgramBuilder<'i> {
    pub fn new_agent_id(&mut self) -> AgentId {
        let a = AgentId(self.next_agent_id);
        self.next_agent_id += 1;
        a
    }
    pub fn get_agent_id(&mut self, n: String) -> AgentId {
        if let Some(id) = self.agent_scope.get(&n) {
            *id
        } else {
            let id = self.new_agent_id();
            self.agent_scope.insert(n, id);
            id
        }
    }
    pub fn immut_get_agent_id(&self, n: String) -> AgentId {
        if let Some(id) = self.agent_scope.get(&n) {
            *id
        } else {
            println!("Agent {} not found", n);
            todo!()
        }
    }
    pub fn get_var_id(&mut self, n: String) -> VarId {
        if let Some(id) = self.vars.var_scope.get(&n) {
            *id
        } else {
            let id = self.vars.net.new_var();
            self.vars.var_scope.insert(n, id);
            id
        }
    }
    pub fn skip_trivia(&mut self) {
        while let Some(c) = self.peek_one() {
            if c.is_ascii_whitespace() {
                self.advance_one();
                continue;
            }
            break;
        }
    }
}
