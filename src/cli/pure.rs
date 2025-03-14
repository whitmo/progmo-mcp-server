use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Args {
    pub command: String,
    pub options: HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Missing command")]
    MissingCommand,
    #[error("Invalid option format: {0}")]
    InvalidOption(String),
    #[error("Missing value for option: {0}")]
    MissingValue(String),
}

impl Args {
    pub fn parse<I>(mut args: I) -> Result<Self, ParseError> 
    where I: Iterator<Item = String> {
        // Skip program name
        args.next();
        
        let command = args.next().ok_or(ParseError::MissingCommand)?;
        let mut options = HashMap::new();
        
        let mut current_key: Option<String> = None;
        
        for arg in args {
            if arg.starts_with("-") {
                // Handle any pending key without value
                if let Some(key) = current_key {
                    return Err(ParseError::MissingValue(key));
                }
                
                let key = arg.trim_start_matches('-').to_string();
                current_key = Some(key);
            } else if let Some(key) = current_key.take() {
                options.insert(key, arg);
            } else {
                return Err(ParseError::InvalidOption(arg));
            }
        }
        
        // Check for trailing key
        if let Some(key) = current_key {
            return Err(ParseError::MissingValue(key));
        }
        
        Ok(Args { command, options })
    }
    
    pub fn get_option(&self, key: &str) -> Option<&str> {
        self.options.get(key).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_command() {
        let args = vec!["program".to_string(), "start".to_string()];
        let parsed = Args::parse(args.into_iter()).unwrap();
        assert_eq!(parsed.command, "start");
        assert!(parsed.options.is_empty());
    }
    
    #[test]
    fn test_parse_with_options() {
        let args = vec![
            "program".to_string(),
            "start".to_string(),
            "-host".to_string(),
            "localhost".to_string(),
            "-port".to_string(),
            "8080".to_string(),
        ];
        let parsed = Args::parse(args.into_iter()).unwrap();
        assert_eq!(parsed.command, "start");
        assert_eq!(parsed.get_option("host"), Some("localhost"));
        assert_eq!(parsed.get_option("port"), Some("8080"));
    }
    
    #[test]
    fn test_parse_missing_value() {
        let args = vec![
            "program".to_string(),
            "start".to_string(),
            "-host".to_string(),
        ];
        let result = Args::parse(args.into_iter());
        assert!(matches!(result, Err(ParseError::MissingValue(_))));
    }
}
