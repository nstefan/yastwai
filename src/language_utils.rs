use anyhow::{Result, anyhow};
use isolang::Language;

/// Language utilities for ISO language code handling
/// 
/// This module provides functions for validating, normalizing, and
/// matching ISO 639-1 (2-letter) and ISO 639-2 (3-letter) language codes.
/// Language code type
pub enum LanguageCodeType {
    /// ISO 639-1 (2-letter) code
    Part1,
    /// ISO 639-2/T (3-letter) code
    Part2T,
    /// ISO 639-2/B (3-letter) code
    Part2B,
}

/// Validate if a language code is a valid ISO 639-1 or ISO 639-2 code
pub fn validate_language_code(code: &str) -> Result<LanguageCodeType> {
    let normalized_code = code.trim().to_lowercase();
    
    // Check for ISO 639-1 (2-letter) code
    if normalized_code.len() == 2 {
        if Language::from_639_1(&normalized_code).is_some() {
            return Ok(LanguageCodeType::Part1);
        }
    } 
    // Check for ISO 639-2 (3-letter) code
    else if normalized_code.len() == 3 {
        // Try to parse as ISO 639-2/T code
        if Language::from_639_3(&normalized_code).is_some() {
            return Ok(LanguageCodeType::Part2T);
        }
        
        // Check if it's a ISO 639-2/B code that differs from ISO 639-2/T
        match normalized_code.as_str() {
            "fre" => return Ok(LanguageCodeType::Part2B), // French (fra in 639-2/T)
            "ger" => return Ok(LanguageCodeType::Part2B), // German (deu in 639-2/T)
            "dut" => return Ok(LanguageCodeType::Part2B), // Dutch (nld in 639-2/T)
            "gre" => return Ok(LanguageCodeType::Part2B), // Greek (ell in 639-2/T)
            "chi" => return Ok(LanguageCodeType::Part2B), // Chinese (zho in 639-2/T)
            "cze" => return Ok(LanguageCodeType::Part2B), // Czech (ces in 639-2/T)
            "ice" => return Ok(LanguageCodeType::Part2B), // Icelandic (isl in 639-2/T)
            "alb" => return Ok(LanguageCodeType::Part2B), // Albanian (sqi in 639-2/T)
            "arm" => return Ok(LanguageCodeType::Part2B), // Armenian (hye in 639-2/T)
            "baq" => return Ok(LanguageCodeType::Part2B), // Basque (eus in 639-2/T)
            "bur" => return Ok(LanguageCodeType::Part2B), // Burmese (mya in 639-2/T)
            "per" => return Ok(LanguageCodeType::Part2B), // Persian (fas in 639-2/T)
            "geo" => return Ok(LanguageCodeType::Part2B), // Georgian (kat in 639-2/T)
            "may" => return Ok(LanguageCodeType::Part2B), // Malay (msa in 639-2/T)
            "mac" => return Ok(LanguageCodeType::Part2B), // Macedonian (mkd in 639-2/T)
            "rum" => return Ok(LanguageCodeType::Part2B), // Romanian (ron in 639-2/T)
            "slo" => return Ok(LanguageCodeType::Part2B), // Slovak (slk in 639-2/T)
            "wel" => return Ok(LanguageCodeType::Part2B), // Welsh (cym in 639-2/T)
            _ => {}
        }
    }
    
    Err(anyhow!("Invalid language code: {}", code))
}

/// Normalize a language code to ISO 639-2/T (3-letter) format
pub fn normalize_to_part2t(code: &str) -> Result<String> {
    let normalized_code = code.trim().to_lowercase();
    
    // If it's a 2-letter code, convert to 3-letter
    if normalized_code.len() == 2 {
        if let Some(lang) = Language::from_639_1(&normalized_code) {
            return Ok(lang.to_639_3().to_string());
        }
    } 
    // If it's already a 3-letter code, ensure it's ISO 639-2/T
    else if normalized_code.len() == 3 {
        // Check if it's already a valid ISO 639-2/T code
        if Language::from_639_3(&normalized_code).is_some() {
            return Ok(normalized_code);
        }
        
        // Check if it's a ISO 639-2/B code that needs converting to ISO 639-2/T
        match normalized_code.as_str() {
            "fre" => return Ok("fra".to_string()),
            "ger" => return Ok("deu".to_string()),
            "dut" => return Ok("nld".to_string()),
            "gre" => return Ok("ell".to_string()),
            "chi" => return Ok("zho".to_string()),
            "cze" => return Ok("ces".to_string()),
            "ice" => return Ok("isl".to_string()),
            "alb" => return Ok("sqi".to_string()),
            "arm" => return Ok("hye".to_string()),
            "baq" => return Ok("eus".to_string()),
            "bur" => return Ok("mya".to_string()),
            "per" => return Ok("fas".to_string()),
            "geo" => return Ok("kat".to_string()),
            "may" => return Ok("msa".to_string()),
            "mac" => return Ok("mkd".to_string()),
            "rum" => return Ok("ron".to_string()),
            "slo" => return Ok("slk".to_string()),
            "wel" => return Ok("cym".to_string()),
            _ => {}
        }
    }
    
    Err(anyhow!("Cannot normalize invalid language code: {}", code))
}

/// Normalize a language code to ISO 639-1 (2-letter) format if possible
/// Falls back to ISO 639-2/T if no ISO 639-1 code exists
pub fn normalize_to_part1_or_part2t(code: &str) -> Result<String> {
    let normalized_code = code.trim().to_lowercase();
    
    // If it's already a 2-letter code, validate it
    if normalized_code.len() == 2 {
        if Language::from_639_1(&normalized_code).is_some() {
            return Ok(normalized_code);
        }
    } 
    // If it's a 3-letter code, try to find corresponding 2-letter code
    else if normalized_code.len() == 3 {
        // First normalize to ISO 639-2/T if it's a ISO 639-2/B code
        let part2t = match normalized_code.as_str() {
            "fre" => "fra",
            "ger" => "deu",
            "dut" => "nld",
            "gre" => "ell",
            "chi" => "zho",
            "cze" => "ces",
            "ice" => "isl",
            "alb" => "sqi",
            "arm" => "hye",
            "baq" => "eus",
            "bur" => "mya",
            "per" => "fas",
            "geo" => "kat",
            "may" => "msa",
            "mac" => "mkd",
            "rum" => "ron",
            "slo" => "slk",
            "wel" => "cym",
            _ => &normalized_code,
        };
        
        // Try to get the language from the ISO 639-2/T code
        if let Some(lang) = Language::from_639_3(part2t) {
            // Try to get the ISO 639-1 code
            if let Some(code_639_1) = lang.to_639_1() {
                return Ok(code_639_1.to_string());
            }
            
            // If no ISO 639-1 code exists, return the ISO 639-2/T code
            return Ok(part2t.to_string());
        }
    }
    
    Err(anyhow!("Cannot normalize invalid language code: {}", code))
}

/// Check if two language codes match (represent the same language)
pub fn language_codes_match(code1: &str, code2: &str) -> bool {
    let normalized1 = match normalize_to_part2t(code1) {
        Ok(n) => n,
        Err(_) => return false,
    };
    
    let normalized2 = match normalize_to_part2t(code2) {
        Ok(n) => n,
        Err(_) => return false,
    };
    
    normalized1 == normalized2
}

/// Get the language name from a code
pub fn get_language_name(code: &str) -> Result<String> {
    let normalized = normalize_to_part2t(code)?;
    let lang = isolang::Language::from_639_3(&normalized)
        .ok_or_else(|| anyhow::anyhow!("Failed to get language from code: {}", normalized))?;
    
    Ok(lang.to_name().to_string())
} 