use crate::builder::ProgramBuilder;
use TSPL::Parser;

#[derive(Debug)]
pub enum Tree {
    Agent(String, Vec<Tree>),
    Var(String),
}

impl<'i> ProgramBuilder<'i> {
    pub fn new(input: &'i str) -> Self {
        let mut a = Self {
            input,
            index: 0,
            def: vec![],
            levels: vec![],
            arities: Default::default(),
            agent_scope_back: Default::default(),
            next_agent_id: 0,
            interaction_system: None,
        };
        a.enter();
        a
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

    pub fn parse_var_name(&mut self) -> Result<String, String> {
        self.skip_trivia();
        let first = self
            .peek_one()
            .ok_or("Expected a character for var_name.")?;
        if first.is_ascii_lowercase() {
            self.advance_one();
            let rest = self.take_while(|c| Self::is_name_character(c));
            return Ok(format!("{}{}", first, rest));
        }
        println!("{:?}", &self.input[self.index..]);
        Err("Expected a variable name.".to_string())
    }

    pub fn parse_ctr_name(&mut self) -> Result<String, String> {
        self.skip_trivia();
        let first = self
            .peek_one()
            .ok_or("Expected a character for ctr_name.")?;
        if first.is_ascii_uppercase() || first.is_ascii_digit() || ".!#$%&/?*-_:;~".contains(first)
        {
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
            self.enter();
            self.skip_trivia();
            self.consume("{")?;
            self.parse_scope()?;
            self.skip_trivia();
            self.consume("}")?;
            let mut result = self.exit().unwrap();
            result.vars.net.system = self.build_interaction_system();

            if false {
                // reduce and show each step
                while let Some((a, b)) = result.vars.net.interactions.pop() {
                    result.vars.net.interact(a, b);
                    let mut vars = result
                        .vars
                        .var_scope
                        .iter()
                        .map(|(k, v)| (v.clone(), k.clone()))
                        .collect();
                    let net = result.vars.net.show_net_compact(
                        &|id| self.agent_scope_back.get(&id).unwrap().to_string(),
                        &mut vars,
                    );
                    println!("{}---", net);
                }
            } else {
                result.vars.net.normal();
                let mut vars = result
                    .vars
                    .var_scope
                    .iter()
                    .map(|(k, v)| (v.clone(), k.clone()))
                    .collect();
                let net = result.vars.net.show_net_compact(
                    &|id| self.agent_scope_back.get(&id).unwrap().to_string(),
                    &mut vars,
                );
                println!("{}", net);
            }
        } else {
            let _ = self.take_while(|c| c != ']'); // Assumes any sequence inside []
        }
        self.consume("]")?;
        Ok(self.index)
    }

    pub fn parse_tree(&mut self) -> Result<Tree, String> {
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
}
