use std::{collections::BTreeMap, rc::Rc};

#[derive(Clone, Debug, Copy, PartialEq, PartialOrd, Ord, Eq)]
pub struct AgentId(pub u64);
#[derive(Clone, Debug, Copy, PartialEq, PartialOrd, Ord, Eq)]
pub struct VarId(pub u64);

#[derive(Clone, Debug)]
pub enum Tree {
    Agent { id: AgentId, aux: Vec<Tree> },
    Var { id: VarId },
}

#[derive(Debug)]
pub struct InteractionRule {
    pub left_ports: Vec<Tree>,
    pub right_ports: Vec<Tree>,
}

#[derive(Debug, Default)]
pub struct InteractionSystem {
    pub rules: BTreeMap<AgentId, BTreeMap<AgentId, InteractionRule>>,
}

impl InteractionSystem {
    pub fn add_rule(&mut self, a: AgentId, b: AgentId, rule: InteractionRule) {
        self.rules.entry(a).or_default().insert(b, rule);
    }
    pub fn get_rule(
        &self,
        a: AgentId,
        b: AgentId,
    ) -> (Option<&InteractionRule>, Option<&InteractionRule>) {
        (
            self.rules.get(&a).and_then(|x| x.get(&b)),
            self.rules.get(&b).and_then(|x| x.get(&a)),
        )
    }
    pub fn has_rule(&self, a: AgentId, b: AgentId) -> bool {
        let (a, b) = self.get_rule(a, b);
        a.or(b).is_some()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Net {
    pub interactions: Vec<(Tree, Tree)>,
    pub vars: BTreeMap<VarId, Option<Tree>>,
    pub stuck: Vec<(Tree, Tree)>,
    pub system: Rc<InteractionSystem>,
}

impl Net {
    pub fn new_var(&mut self) -> VarId {
        let mut next_key = 0;
        for i in 0u64.. {
            if !self.vars.contains_key(&VarId(i)) {
                next_key = i;
                break;
            }
        }
        let next_key = VarId(next_key);
        assert!(self.vars.insert(next_key, None).is_none());
        next_key
    }
    fn link(&mut self, a: Tree, b: Tree) {
        self.interactions.push((a, b))
    }
    fn freshen(&mut self, scope: &mut BTreeMap<VarId, VarId>, tree: &Tree) -> Tree {
        use Tree::*;
        match tree {
            Agent { id, aux } => Agent {
                id: *id,
                aux: aux.into_iter().map(|x| self.freshen(scope, x)).collect(),
            },
            Var { id } => match scope.remove(id) {
                Some(e) => Var { id: e },
                None => {
                    let new_id = self.new_var();
                    scope.insert(*id, new_id);
                    Var { id: new_id }
                }
            },
        }
    }
    fn apply_rule(&mut self, rule: &InteractionRule, left: Vec<Tree>, right: Vec<Tree>) {
        let mut var_set = BTreeMap::new();
        for (i, j) in rule
            .left_ports
            .iter()
            .zip(left.into_iter())
            .chain(rule.right_ports.iter().zip(right.into_iter()))
        {
            let i = self.freshen(&mut var_set, i);
            self.link(i, j);
        }
    }
    pub fn interact(&mut self, a: Tree, b: Tree) {
        use Tree::*;
        match (a, b) {
            (Agent { id: id1, aux: aux1 }, Agent { id: id2, aux: aux2 }) => {
                let rules = self.system.clone();
                let (rule, rule_flip) = rules.get_rule(id1, id2);
                if let Some(r) = rule {
                    self.apply_rule(r, aux1, aux2);
                } else if let Some(r) = rule_flip {
                    self.apply_rule(r, aux2, aux1);
                } else {
                    self.stuck
                        .push((Agent { id: id1, aux: aux1 }, Agent { id: id2, aux: aux2 }));
                }
            }
            (a, Var { id }) | (Var { id }, a) => {
                if let Some(b) = self.vars.get_mut(&id).unwrap().take() {
                    self.vars.remove(&id);
                    self.link(a, b)
                } else {
                    *self.vars.get_mut(&id).unwrap() = Some(a);
                }
            }
        }
    }
    pub fn normal(&mut self) {
        while let Some((a, b)) = self.interactions.pop() {
            self.interact(a, b)
        }
    }

    pub fn show_net_compact(
        &self,
        show_agent: &dyn Fn(AgentId) -> String,
        scope: &mut BTreeMap<VarId, String>,
    ) -> String {
        self.show_net(show_agent, scope, false)
    }
    pub fn show_net(
        &self,
        show_agent: &dyn Fn(AgentId) -> String,
        scope: &mut BTreeMap<VarId, String>,
        show_vars: bool,
    ) -> String {
        let mut visited = vec![];
        use std::fmt::Write;
        let mut s = String::new();
        for (a, b) in &self.interactions {
            write!(
                &mut s,
                "{} = {}\n",
                self.show_tree(show_agent, scope, &mut visited, &a),
                self.show_tree(show_agent, scope, &mut visited, &b)
            )
            .unwrap();
        }
        if show_vars {
            for (a, b) in &self.vars {
                if visited.contains(a) {
                    continue;
                }
                if let Some(b) = b {
                    visited.push(*a);
                    let a = scope.get(a).cloned().unwrap_or(format!("?{}", a.0));
                    write!(
                        &mut s,
                        "{} = {}\n",
                        a,
                        self.show_tree(show_agent, scope, &mut visited, &b)
                    )
                    .unwrap();
                }
            }
        }
        for (a, b) in &self.stuck {
            write!(
                &mut s,
                "{} = {}\n",
                self.show_tree(show_agent, scope, &mut visited, &a),
                self.show_tree(show_agent, scope, &mut visited, &b)
            )
            .unwrap();
        }
        s
    }
    pub fn show_tree(
        &self,
        show_agent: &dyn Fn(AgentId) -> String,
        scope: &mut BTreeMap<VarId, String>,
        visited: &mut Vec<VarId>,
        tree: &Tree,
    ) -> String {
        match tree {
            Tree::Agent { id, aux } => {
                use std::fmt::Write;
                let mut s = String::new();
                write!(&mut s, "{}", show_agent(*id)).unwrap();
                let mut i = aux.iter();
                if let Some(e) = i.next() {
                    write!(&mut s, "(").unwrap();
                    write!(&mut s, "{}", self.show_tree(show_agent, scope, visited, e)).unwrap();
                    for subtree in i {
                        write!(
                            &mut s,
                            " {}",
                            self.show_tree(show_agent, scope, visited, subtree)
                        )
                        .unwrap();
                    }
                    write!(&mut s, ")").unwrap();
                }
                s
            }
            Tree::Var { id } => {
                if !visited.contains(id)
                    && let Some(Some(b)) = self.vars.get(id)
                {
                    visited.push(*id);
                    self.show_tree(show_agent, scope, visited, b)
                } else {
                    scope.get(id).cloned().unwrap_or(format!("?{}", id.0))
                }
            }
        }
    }
    pub fn substitute_ref(&self, tree: &Tree) -> Tree {
        match tree {
            Tree::Agent { id, aux } => Tree::Agent {
                id: *id,
                aux: aux.into_iter().map(|x| self.substitute_ref(x)).collect(),
            },
            Tree::Var { id } => {
                if let Some(Some(b)) = self.vars.get(id) {
                    self.substitute_ref(b)
                } else {
                    Tree::Var { id: *id }
                }
            }
        }
    }
    pub fn substitute(&mut self, tree: Tree) -> Tree {
        match tree {
            Tree::Agent { id, aux } => Tree::Agent {
                id,
                aux: aux.into_iter().map(|x| self.substitute(x)).collect(),
            },
            Tree::Var { id } => {
                if let Some(b) = self.vars.get_mut(&id).unwrap().take() {
                    self.vars.remove(&id);
                    self.substitute(b)
                } else {
                    Tree::Var { id }
                }
            }
        }
    }
}
