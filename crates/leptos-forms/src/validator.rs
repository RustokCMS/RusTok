use regex::Regex;
use std::sync::Arc;

#[derive(Clone)]
pub struct Validator {
    rules: Vec<ValidationRule>,
}

#[derive(Clone)]
enum ValidationRule {
    Required,
    Email,
    MinLength(usize),
    MaxLength(usize),
    Pattern(Arc<Regex>),
    Custom(Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>),
}

impl Validator {
    pub fn new() -> Self {
        Self { rules: vec![] }
    }
    
    pub fn required(mut self) -> Self {
        self.rules.push(ValidationRule::Required);
        self
    }
    
    pub fn email(mut self) -> Self {
        self.rules.push(ValidationRule::Email);
        self
    }
    
    pub fn min_length(mut self, len: usize) -> Self {
        self.rules.push(ValidationRule::MinLength(len));
        self
    }
    
    pub fn max_length(mut self, len: usize) -> Self {
        self.rules.push(ValidationRule::MaxLength(len));
        self
    }
    
    pub fn pattern(mut self, pattern: &str) -> Result<Self, regex::Error> {
        let regex = Regex::new(pattern)?;
        self.rules.push(ValidationRule::Pattern(Arc::new(regex)));
        Ok(self)
    }
    
    pub fn custom<F>(mut self, validator: F) -> Self
    where
        F: Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    {
        self.rules.push(ValidationRule::Custom(Arc::new(validator)));
        self
    }
    
    pub fn validate(&self, value: &str) -> Result<(), String> {
        for rule in &self.rules {
            match rule {
                ValidationRule::Required => {
                    if value.trim().is_empty() {
                        return Err("This field is required".to_string());
                    }
                }
                ValidationRule::Email => {
                    let email_regex = Regex::new(
                        r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
                    ).unwrap();
                    if !email_regex.is_match(value) {
                        return Err("Invalid email address".to_string());
                    }
                }
                ValidationRule::MinLength(len) => {
                    if value.len() < *len {
                        return Err(format!("Must be at least {} characters", len));
                    }
                }
                ValidationRule::MaxLength(len) => {
                    if value.len() > *len {
                        return Err(format!("Must be at most {} characters", len));
                    }
                }
                ValidationRule::Pattern(regex) => {
                    if !regex.is_match(value) {
                        return Err("Invalid format".to_string());
                    }
                }
                ValidationRule::Custom(validator) => {
                    validator(value)?;
                }
            }
        }
        Ok(())
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}
