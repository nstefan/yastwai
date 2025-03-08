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
    /// Unknown or invalid code
    #[allow(dead_code)]
    Unknown,
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
        // This is a bit tricky since isolang doesn't directly support ISO 639-2/B
        // We'll check some common cases
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
        if let Some(_) = Language::from_639_1(&normalized_code) {
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
    let normalized_code = code.trim().to_lowercase();
    
    // Try as ISO 639-1 code
    if normalized_code.len() == 2 {
        if let Some(lang) = Language::from_639_1(&normalized_code) {
            return Ok(lang.to_name().to_string());
        }
    }
    
    // Try as ISO 639-2/T code
    if normalized_code.len() == 3 {
        if let Some(lang) = Language::from_639_3(&normalized_code) {
            return Ok(lang.to_name().to_string());
        }
        
        // Check if it's a ISO 639-2/B code
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
            _ => "",
        };
        
        if !part2t.is_empty() {
            if let Some(lang) = Language::from_639_3(part2t) {
                return Ok(lang.to_name().to_string());
            }
        }
    }
    
    Err(anyhow!("Could not find language name for code: {}", code))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_config::SubtitleInfo;
    
    #[test]
    fn test_validate_language_code() {
        // ISO 639-1 tests
        assert!(matches!(validate_language_code("en").unwrap(), LanguageCodeType::Part1));
        assert!(matches!(validate_language_code("fr").unwrap(), LanguageCodeType::Part1));
        assert!(matches!(validate_language_code("de").unwrap(), LanguageCodeType::Part1));
        
        // ISO 639-2/T tests
        assert!(matches!(validate_language_code("eng").unwrap(), LanguageCodeType::Part2T));
        assert!(matches!(validate_language_code("fra").unwrap(), LanguageCodeType::Part2T));
        assert!(matches!(validate_language_code("deu").unwrap(), LanguageCodeType::Part2T));
        
        // ISO 639-2/B tests
        assert!(matches!(validate_language_code("fre").unwrap(), LanguageCodeType::Part2B));
        assert!(matches!(validate_language_code("ger").unwrap(), LanguageCodeType::Part2B));
        
        // Whitespace and case tests
        assert!(matches!(validate_language_code(" EN ").unwrap(), LanguageCodeType::Part1));
        assert!(matches!(validate_language_code("ENG").unwrap(), LanguageCodeType::Part2T));
        
        // Invalid codes
        assert!(validate_language_code("xyz").is_err());
        assert!(validate_language_code("123").is_err());
        assert!(validate_language_code("e").is_err());
    }
    
    #[test]
    fn test_normalize_to_part2t() {
        assert_eq!(normalize_to_part2t("en").unwrap(), "eng");
        assert_eq!(normalize_to_part2t("fr").unwrap(), "fra");
        assert_eq!(normalize_to_part2t("eng").unwrap(), "eng");
        assert_eq!(normalize_to_part2t("fra").unwrap(), "fra");
        assert_eq!(normalize_to_part2t("fre").unwrap(), "fra");
        assert_eq!(normalize_to_part2t("ger").unwrap(), "deu");
        
        // Case insensitivity
        assert_eq!(normalize_to_part2t("EN").unwrap(), "eng");
        assert_eq!(normalize_to_part2t("FRE").unwrap(), "fra");
        
        // Whitespace
        assert_eq!(normalize_to_part2t(" en ").unwrap(), "eng");
    }
    
    #[test]
    fn test_language_codes_match() {
        assert!(language_codes_match("en", "eng"));
        assert!(language_codes_match("eng", "en"));
        assert!(language_codes_match("eng", "eng"));
        assert!(language_codes_match("fr", "fra"));
        assert!(language_codes_match("fr", "fre"));
        assert!(language_codes_match("fra", "fre"));
        
        // Case insensitivity
        assert!(language_codes_match("EN", "eng"));
        assert!(language_codes_match("EN", "ENG"));
        
        // Whitespace
        assert!(language_codes_match(" en ", "eng"));
        
        // Non-matches
        assert!(!language_codes_match("en", "fra"));
        assert!(!language_codes_match("eng", "fre"));
    }
    
    #[test]
    fn test_get_language_name() {
        assert_eq!(get_language_name("en").unwrap(), "English");
        assert_eq!(get_language_name("eng").unwrap(), "English");
        assert_eq!(get_language_name("fr").unwrap(), "French");
        assert_eq!(get_language_name("fra").unwrap(), "French");
        assert_eq!(get_language_name("fre").unwrap(), "French");
        
        // Invalid codes
        assert!(get_language_name("xyz").is_err());
    }
    
    #[test]
    fn test_subtitle_track_selection_with_iso_codes() {
        // Create a test function to simulate subtitle track selection with ISO codes
        // This is testing from the perspective of the language_utils module, not directly
        // calling the subtitle_processor's select_subtitle_track
        
        // Create mock subtitle tracks with various language codes
        let tracks = vec![
            SubtitleInfo {
                index: 0,
                codec_name: "subrip".to_string(),
                language: Some("eng".to_string()),  // ISO 639-2/T
                title: None,
            },
            SubtitleInfo {
                index: 1,
                codec_name: "subrip".to_string(),
                language: Some("fre".to_string()),  // ISO 639-2/B
                title: None,
            },
            SubtitleInfo {
                index: 2,
                codec_name: "subrip".to_string(),
                language: Some("de".to_string()),  // ISO 639-1
                title: None,
            },
            SubtitleInfo {
                index: 3,
                codec_name: "subrip".to_string(),
                language: Some("ita".to_string()),  // ISO 639-2/T
                title: None,
            },
            SubtitleInfo {
                index: 4,
                codec_name: "subrip".to_string(),
                language: None,  // Unknown language
                title: Some("English Commentary".to_string()),
            },
        ];
        
        // Test matching using ISO 639-1 code
        let track_matches = tracks.iter()
            .filter(|track| {
                if let Some(lang) = &track.language {
                    language_codes_match(lang, "en")
                } else {
                    false
                }
            })
            .map(|track| track.index)
            .collect::<Vec<_>>();
        
        assert_eq!(track_matches, vec![0]);  // Should match "eng" with "en"
        
        // Test matching using ISO 639-2/T code
        let track_matches = tracks.iter()
            .filter(|track| {
                if let Some(lang) = &track.language {
                    language_codes_match(lang, "fra")
                } else {
                    false
                }
            })
            .map(|track| track.index)
            .collect::<Vec<_>>();
        
        assert_eq!(track_matches, vec![1]);  // Should match "fre" with "fra"
        
        // Test matching using ISO 639-2/B code
        let track_matches = tracks.iter()
            .filter(|track| {
                if let Some(lang) = &track.language {
                    language_codes_match(lang, "fre")
                } else {
                    false
                }
            })
            .map(|track| track.index)
            .collect::<Vec<_>>();
        
        assert_eq!(track_matches, vec![1]);  // Should match "fre" with "fre"
        
        // Test matching when track has ISO 639-1 code but query is ISO 639-2
        let track_matches = tracks.iter()
            .filter(|track| {
                if let Some(lang) = &track.language {
                    language_codes_match(lang, "deu")
                } else {
                    false
                }
            })
            .map(|track| track.index)
            .collect::<Vec<_>>();
        
        assert_eq!(track_matches, vec![2]);  // Should match "de" with "deu"
    }
} 