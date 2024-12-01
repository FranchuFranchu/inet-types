use std::{collections::BTreeMap, rc::Rc};

use TSPL::Parser;

use crate::run::{AgentId, InteractionSystem, Net, Tree, VarId};

#[derive(Debug, Clone)]
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

#[derive(Debug, Default)]
pub struct Level {
    pub agent_scope: BTreeMap<String, AgentId>,
    pub vars: Vars,
}

#[derive(Debug)]
pub struct ProgramBuilder<'i> {
    pub input: &'i str,
    pub index: usize,
    pub levels: Vec<Level>,
    pub def: Vec<Definition>,
    pub agent_scope_back: BTreeMap<AgentId, String>,
    pub arities: BTreeMap<AgentId, u64>,
    pub next_agent_id: u64,
    pub interaction_system: Option<Rc<InteractionSystem>>,
}
impl<'i> ProgramBuilder<'i> {
    pub fn agent_list<'a>(&'a self) -> impl Iterator<Item = AgentId> + 'a {
        self.levels
            .iter()
            .map(|x| x.agent_scope.values())
            .flatten()
            .map(|x| *x)
            .chain(
                self.levels
                    .iter()
                    .map(|x| x.agent_scope.values())
                    .flatten()
                    .map(|x| self.agent_inverse(*x)),
            )
    }
    pub fn new_agent_id(&mut self) -> AgentId {
        let a = AgentId(self.next_agent_id, 0);
        self.next_agent_id += 1;
        a
    }
    pub fn show_agent(&self, AgentId(a, b): AgentId) -> String {
        if b > 0 {
            format!("~{}", self.show_agent(AgentId(a, b - 1)))
        } else {
            self.agent_scope_back
                .get(&AgentId(a, b))
                .unwrap()
                .to_string()
        }
    }
    pub fn get_or_new_agent_id(&mut self, n: String) -> AgentId {
        if n.starts_with("~") && &n != "~" {
            let s = &n[1..];
            let id = self.get_or_new_agent_id(s.to_string());
            let id = AgentId(id.0, id.1 + 1);
            self.agent_scope_back.insert(id, n);
            id
        } else {
            if let Some(id) = self.get_agent_id(&n) {
                id
            } else {
                let id = self.new_agent_id();
                self.levels
                    .last_mut()
                    .unwrap()
                    .agent_scope
                    .insert(n.clone(), id);
                self.agent_scope_back.insert(id, n.clone());
                self.agent_scope_back
                    .insert(self.agent_inverse(id), "~".to_string() + &n);
                id
            }
        }
    }
    pub fn set_arity(&mut self, a: AgentId, arity: u64) {
        self.arities.insert(a, arity);
        let inv = self.agent_inverse(a);
        self.arities.insert(inv, arity);
    }
    pub fn get_agent_id(&self, n: &str) -> Option<AgentId> {
        for level in &self.levels {
            if let Some(a) = level.agent_scope.get(n) {
                return Some(*a);
            }
        }
        return None;
    }
    pub fn get_var_id(&self, n: &str) -> Option<VarId> {
        for level in &self.levels {
            if let Some(a) = level.vars.var_scope.get(n) {
                return Some(*a);
            }
        }
        return None;
    }
    pub fn get_or_new_var_id(&mut self, n: String) -> VarId {
        if let Some(id) = self.get_var_id(&n) {
            id
        } else {
            let id = self.levels.last_mut().unwrap().vars.net.new_var();
            self.levels.last_mut().unwrap().vars.var_scope.insert(n, id);
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
    pub fn agent_inverse(&self, AgentId(a, b): AgentId) -> AgentId {
        AgentId(a, (b + 1) % 2)
    }
}
