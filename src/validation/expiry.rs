use chrono::{NaiveDate, Local};
use crate::models::{VisualData, ExpiryValidationResult, ValidationIssue, ValidationIssueType};
use crate::utils::PassportError;

pub struct ExpiryValidator;

impl ExpiryValidator {
    pub fn validate(visual_data: &VisualData) -> Result<ExpiryValidationResult, PassportError> {
        let mut issues = Vec::new();
        let mut not_expired = false;
        
        // Parse the expiry date
        if let Some(date) = Self::parse_date(&visual_data.date_of_expiry) {
            // Get current date
            let today = Local::now().naive_local().date();
            
            // Check if passport is expired
            not_expired = date >= today;
            
            if !not_expired {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::Expiry,
                    message: "Passport has expired".to_string(),
                });
            }
        } else {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Expiry,
                message: "Invalid expiry date format".to_string(),
            });
        }
        
        Ok(ExpiryValidationResult {
            is_valid: not_expired,
            not_expired,
            issues,
        })
    }
    
    // Parse a date string in the format "DD MM YYYY"
    fn parse_date(date_str: &str) -> Option<NaiveDate> {
        let parts: Vec<&str> = date_str.split_whitespace().collect();
        if parts.len() != 3 {
            return None;
        }
        
        let day = parts[0].parse::<u32>().ok()?;
        let month = parts[1].parse::<u32>().ok()?;
        let year = parts[2].parse::<i32>().ok()?;
        
        NaiveDate::from_ymd_opt(year, month, day)
    }
}
