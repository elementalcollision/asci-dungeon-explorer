use super::StateType;
use std::collections::VecDeque;

pub struct StateStack {
    states: VecDeque<StateType>,
}

impl StateStack {
    pub fn new() -> Self {
        let mut stack = StateStack {
            states: VecDeque::new(),
        };
        stack.states.push_back(StateType::MainMenu);
        stack
    }
    
    pub fn current(&self) -> StateType {
        *self.states.back().unwrap_or(&StateType::MainMenu)
    }
    
    pub fn push(&mut self, state: StateType) {
        self.states.push_back(state);
    }
    
    pub fn pop(&mut self) -> Option<StateType> {
        if self.states.len() > 1 {
            self.states.pop_back()
        } else {
            None
        }
    }
    
    pub fn replace(&mut self, state: StateType) {
        if !self.states.is_empty() {
            self.states.pop_back();
        }
        self.states.push_back(state);
    }
    
    pub fn clear(&mut self) {
        self.states.clear();
        self.states.push_back(StateType::MainMenu);
    }
}