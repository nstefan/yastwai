/*!
 * Tests for language utility functions
 */

use yastwai::language_utils::{normalize_to_part2t, language_codes_match, get_language_name};
use yastwai::app_config::SubtitleInfo;

/// Test normalization of language codes to ISO 639-2/T format
#[test]
fn test_normalize_to_part2t_withValidCodes_shouldNormalizeCorrectly() {
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

/// Test matching of different language code formats
#[test]
fn test_language_codes_match_withMatchingCodes_shouldReturnTrue() {
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

/// Test retrieval of language names from codes
#[test]
fn test_get_language_name_withValidCodes_shouldReturnCorrectName() {
    assert_eq!(get_language_name("en").unwrap(), "English");
    assert_eq!(get_language_name("eng").unwrap(), "English");
    assert_eq!(get_language_name("fr").unwrap(), "French");
    assert_eq!(get_language_name("fra").unwrap(), "French");
    assert_eq!(get_language_name("fre").unwrap(), "French");
    
    // Invalid codes
    assert!(get_language_name("xyz").is_err());
}

/// Test subtitle track selection with different ISO code formats
#[test]
fn test_subtitle_track_selection_withIsoCodes_shouldMatchCorrectly() {
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
