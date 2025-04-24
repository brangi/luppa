use crate::utils::PassportError;

#[allow(dead_code)]
pub struct CountryRules {
    #[allow(dead_code)]
    pub countries: Vec<CountryRule>,
}

#[allow(dead_code)]
pub struct CountryRule {
    pub country_code: String,
    pub country_name: String,
    pub document_number_format: String,
    pub validation_rules: Vec<ValidationRule>,
}

#[allow(dead_code)]
pub enum ValidationRule {
    RequiredField(String),
    FieldFormat(String, String), // Field name, Regex pattern
    DateFormat(String, String),  // Field name, Format pattern
    FieldLength(String, usize),  // Field name, Expected length
    Custom(String, String),      // Rule name, Description
}

impl CountryRules {
    pub fn new() -> Self {
        let mut countries = Vec::new();
        
        // Add rule for Mexico (MEX)
        countries.push(CountryRule {
            country_code: "MEX".to_string(),
            country_name: "MEXICO".to_string(),
            document_number_format: r"^[A-Z0-9]{8,9}$".to_string(),
            validation_rules: vec![
                ValidationRule::RequiredField("document_number".to_string()),
                ValidationRule::RequiredField("surname".to_string()),
                ValidationRule::RequiredField("given_names".to_string()),
                ValidationRule::DateFormat("date_of_birth".to_string(), "DD MM YYYY".to_string()),
                ValidationRule::DateFormat("date_of_expiry".to_string(), "DD MM YYYY".to_string()),
            ],
        });
        
        CountryRules { countries }
    }
    
    #[allow(dead_code)]
    pub fn get_rule(&self, country_code: &str) -> Result<&CountryRule, PassportError> {
        self.countries.iter()
            .find(|rule| rule.country_code == country_code)
            .ok_or_else(|| PassportError::CountryRuleNotFound(format!("No rule found for country code: {}", country_code)))
    }
}
