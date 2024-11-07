use std::collections::BTreeMap;

use crate::{builder::ProgramBuilder, run::AgentId};
use TSPL::Parser;

#[derive(Debug)]
pub enum Tree {
    Agent(String, Vec<Tree>),
    Var(String),
}

#[derive(Debug)]
pub struct Redex {
    pub left_tree: Tree,
    pub right_tree: Tree,
}

#[derive(Debug)]
pub enum Item {
    Definition(Tree, Tree, Vec<Redex>),
}

#[derive(Debug)]
pub struct Book {
    pub items: Vec<Item>,
}

impl<'i> ProgramBuilder<'i> {
    pub fn new(input: &'i str) -> Self {
        Self {
            input,
            index: 0,
            def: vec![],
            vars: Default::default(),
            agent_scope: Default::default(),
            next_agent_id: 0,
            interaction_system: None,
        }
    }
}

impl<'i> Parser<'i> for ProgramBuilder<'i> {
    fn input(&mut self) -> &'i str {
        &self.input
    }
    fn index(&mut self) -> &mut usize {
        &mut self.index
    }
}

impl<'i> ProgramBuilder<'i> {
    fn is_name_character(c: char) -> bool {
        c.is_ascii_alphanumeric() || ".!#$%&/?*-_:;".contains(c)
    }

    fn parse_var_name(&mut self) -> Result<String, String> {
        self.skip_trivia();
        let first = self
            .peek_one()
            .ok_or("Expected a character for var_name.")?;
        if first.is_ascii_lowercase() {
            self.advance_one();
            let rest = self.take_while(|c| Self::is_name_character(c));
            return Ok(format!("{}{}", first, rest));
        }
        Err("Expected a variable name.".to_string())
    }

    fn parse_ctr_name(&mut self) -> Result<String, String> {
        self.skip_trivia();
        let first = self
            .peek_one()
            .ok_or("Expected a character for ctr_name.")?;
        if first.is_ascii_uppercase() || first.is_ascii_digit() || ".!#$%&/?*-_:;".contains(first) {
            self.advance_one();
            let rest = self.take_while(|c| Self::is_name_character(c));
            Ok(format!("{}{}", first, rest))
        } else {
            Err("Expected an agent name.".to_string())
        }
    }

    pub fn parse_macro(&mut self) -> Result<usize, String> {
        let ctr_name = self.parse_ctr_name()?;
        self.consume("[")?;
        if ctr_name == "Reduce" {
            self.vars = Default::default();
            let redexes = self.parse_redex_list()?;
            for i in redexes {
                let interaction = (
                    self.tree_ast_to_rt(i.left_tree),
                    self.tree_ast_to_rt(i.right_tree),
                );
                self.vars.net.interactions.push(interaction);
            }
            self.vars.net.system = self.build_interaction_system();

            if false {
                // reduce and show each step
                while let Some((a, b)) = self.vars.net.interactions.pop() {
                    self.vars.net.interact(a, b);
                    let mut vars = self
                        .vars
                        .var_scope
                        .iter()
                        .map(|(k, v)| (v.clone(), k.clone()))
                        .collect();
                    let agents: BTreeMap<AgentId, String> = self
                        .agent_scope
                        .iter()
                        .map(|(k, v)| (v.clone(), k.clone()))
                        .collect();
                    let net = self
                        .vars
                        .net
                        .show_net_compact(&|id| agents.get(&id).unwrap().to_string(), &mut vars);
                    println!("{}---", net);
                }
            } else {
                self.vars.net.normal();
                let mut vars = self
                    .vars
                    .var_scope
                    .iter()
                    .map(|(k, v)| (v.clone(), k.clone()))
                    .collect();
                let agents: BTreeMap<AgentId, String> = self
                    .agent_scope
                    .iter()
                    .map(|(k, v)| (v.clone(), k.clone()))
                    .collect();
                let net = self
                    .vars
                    .net
                    .show_net_compact(&|id| agents.get(&id).unwrap().to_string(), &mut vars);
                println!("{}", net);
            }
        } else {
            let _ = self.take_while(|c| c != ']'); // Assumes any sequence inside []
        }
        self.consume("]")?;
        Ok(self.index)
    }

    fn parse_tree(&mut self) -> Result<Tree, String> {
        self.skip_trivia();
        let mut old_idx = self.index;
        while let Ok(m) = self.parse_macro() {
            old_idx = m;
        }
        self.index = old_idx;
        let old_idx = self.index;
        if let Ok(ctr_name) = self.parse_ctr_name() {
            self.skip_trivia();
            if self.peek_one() == Some('(') {
                self.consume("(")?;
                let mut args = vec![];
                while self.peek_one() != Some(')') {
                    args.push(self.parse_tree()?);
                    self.skip_trivia();
                }
                self.consume(")")?;
                return Ok(Tree::Agent(ctr_name, args));
            }
            return Ok(Tree::Agent(ctr_name, vec![]));
        } else {
            self.index = old_idx;
        }
        let var_name = self.parse_var_name()?;
        Ok(Tree::Var(var_name))
    }

    fn parse_redex(&mut self) -> Result<Redex, String> {
        let left_tree = self.parse_tree()?;
        self.consume("=")?;
        let right_tree = self.parse_tree()?;
        Ok(Redex {
            left_tree,
            right_tree,
        })
    }
    pub fn parse_redex_list(&mut self) -> Result<Vec<Redex>, String> {
        self.consume("{")?;
        let mut redexes = vec![];
        while self.peek_one() != Some('}') {
            redexes.push(self.parse_redex()?);
            self.skip_trivia();
        }
        self.consume("}")?;
        Ok(redexes)
    }
    pub fn parse_item(&mut self) -> Result<Item, String> {
        let mut old_idx = self.index;
        while let Ok(m) = self.parse_macro() {
            old_idx = m;
        }
        self.index = old_idx;
        let tree = self.parse_tree()?;
        self.consume("=")?;
        let right_tree = self.parse_tree()?;
        self.skip_trivia();
        if self.peek_one() == Some('{') {
            let redexes = self.parse_redex_list()?;
            return Ok(Item::Definition(tree, right_tree, redexes));
        }
        Ok(Item::Definition(tree, right_tree, vec![]))
    }

    pub fn parse_book(&mut self) -> Result<Vec<Item>, String> {
        self.skip_trivia();
        let mut book = vec![];
        while self.peek_one().is_some() {
            book.push(self.parse_item()?);
            self.skip_trivia();
        }
        Ok(book)
    }
}
