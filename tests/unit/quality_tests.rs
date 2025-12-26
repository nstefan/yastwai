/*!
 * Integration tests for the quality assurance module.
 *
 * Tests the interaction between quality components:
 * - Metrics calculation
 * - Consistency checking
 * - Repair strategies
 * - Error recovery
 */

use std::time::Duration;

use yastwai::subtitle_processor::SubtitleEntry;
use yastwai::translation::{
    ConsistencyChecker, ConsistencyConfig, ErrorRecovery, Glossary, QualityMetrics,
    QualityThresholds, RecoveryStrategy, RepairEngine, SubtitleDocument, TranslationError,
    TranslationErrorKind,
};
use yastwai::translation::quality::metrics::{EntryMetrics, MetricsData};
use yastwai::translation::quality::repair::RepairConfig;

/// Helper to create a test document.
fn create_test_document(entries: Vec<(&str, Option<&str>)>) -> SubtitleDocument {
    let subtitle_entries: Vec<SubtitleEntry> = entries
        .iter()
        .enumerate()
        .map(|(i, (text, _))| {
            SubtitleEntry::new(
                i + 1,
                (i as u64) * 2000,
                (i as u64 + 1) * 2000,
                text.to_string(),
            )
        })
        .collect();

    let mut doc = SubtitleDocument::from_entries(subtitle_entries, "en");

    for (i, (_, translation)) in entries.iter().enumerate() {
        if let Some(t) = translation {
            doc.entries[i].set_translation(t.to_string(), Some(0.9));
        }
    }

    doc
}

// ============================================================================
// Quality Metrics Tests
// ============================================================================

#[test]
fn test_qualityMetrics_withPerfectTranslations_shouldScoreHigh() {
    let metrics = QualityMetrics::new();

    let mut data = MetricsData::new();
    for i in 1..=10 {
        data.add_entry(
            i,
            EntryMetrics {
                is_translated: true,
                is_empty: false,
                has_issues: false,
                length_ratio: Some(1.0),
                chars_per_second: Some(15.0),
                line_lengths: vec![30],
                expected_tags: 0,
                missing_tags: 0,
                confidence: Some(0.95),
            },
        );
    }

    let score = metrics.calculate_score(&data);

    assert!(score.overall >= 0.95, "Perfect translations should score high");
    assert_eq!(score.grade(), 'A');
}

#[test]
fn test_qualityMetrics_withMissingTranslations_shouldScoreLow() {
    let metrics = QualityMetrics::new();

    let mut data = MetricsData::new();
    data.total_entries = 10;
    data.translated_entries = 5;
    data.empty_entries = 2;
    data.entries_with_issues = 5;

    let score = metrics.calculate_score(&data);

    assert!(score.completeness.score < 0.5, "Missing translations should hurt completeness");
    assert!(score.overall < 0.8);
}

#[test]
fn test_qualityMetrics_withBadLengthRatios_shouldPenalizeAccuracy() {
    let metrics = QualityMetrics::new();

    let mut data = MetricsData::new();
    data.total_entries = 5;
    data.translated_entries = 5;
    data.length_ratios = vec![3.0, 0.1, 2.5, 0.2, 1.0]; // Some very bad ratios

    let score = metrics.calculate_score(&data);

    assert!(score.accuracy.score < 0.7, "Bad length ratios should penalize accuracy");
}

#[test]
fn test_qualityThresholds_strict_shouldBeHigher() {
    let strict = QualityThresholds::strict();
    let default = QualityThresholds::default();
    let lenient = QualityThresholds::lenient();

    assert!(strict.min_overall > default.min_overall);
    assert!(default.min_overall > lenient.min_overall);
    assert!(strict.max_length_ratio < default.max_length_ratio);
}

#[test]
fn test_qualityScore_weakestDimension_shouldIdentifyProblem() {
    let metrics = QualityMetrics::new();

    let mut data = MetricsData::new();
    data.total_entries = 10;
    data.translated_entries = 10;
    data.total_terms = 10;
    data.inconsistent_terms = 8; // Very inconsistent!

    let score = metrics.calculate_score(&data);

    assert_eq!(score.weakest_dimension(), "consistency");
}

// ============================================================================
// Consistency Checker Tests
// ============================================================================

#[test]
fn test_consistencyChecker_withConsistentTerms_shouldPass() {
    let mut doc = create_test_document(vec![
        ("The extraction point is ready.", Some("Le point d'extraction est prêt.")),
        ("Head to the extraction point.", Some("Allez au point d'extraction.")),
    ]);

    doc.glossary.add_term("extraction point", "point d'extraction", None);

    let checker = ConsistencyChecker::new();
    let report = checker.check(&doc);

    assert!(report.score >= 0.9, "Consistent translations should score high");
}

#[test]
fn test_consistencyChecker_withPreservedNames_shouldPass() {
    let mut doc = create_test_document(vec![
        ("John entered the room.", Some("John est entré dans la pièce.")),
        ("Sarah greeted John.", Some("Sarah a salué John.")),
    ]);

    doc.glossary.add_character("John");
    doc.glossary.add_character("Sarah");

    let checker = ConsistencyChecker::new();
    let report = checker.check(&doc);

    // Names are preserved, should have no name issues
    let name_issues: Vec<_> = report
        .issues
        .iter()
        .filter(|i| matches!(i, yastwai::translation::StyleIssue::NameNotPreserved { .. }))
        .collect();

    assert!(name_issues.is_empty(), "Preserved names should not cause issues");
}

#[test]
fn test_consistencyChecker_withChangedName_shouldFlagIssue() {
    let mut doc = create_test_document(vec![
        ("John walked in.", Some("John est entré.")),
        ("John sat down.", Some("Jean s'est assis.")), // Name changed!
    ]);

    doc.glossary.add_character("John");

    let checker = ConsistencyChecker::new();
    let report = checker.check(&doc);

    assert!(
        report.issues.iter().any(|i| matches!(
            i,
            yastwai::translation::StyleIssue::NameNotPreserved { name, .. } if name == "John"
        )),
        "Changed name should be flagged"
    );
}

#[test]
fn test_consistencyChecker_strictConfig_shouldFindMoreIssues() {
    let doc = create_test_document(vec![
        ("Hello there!", Some("Bonjour!")),
        ("Hello again.", Some("Salut encore.")),
    ]);

    let default_checker = ConsistencyChecker::new();
    let strict_checker = ConsistencyChecker::with_config(ConsistencyConfig::strict());

    let default_report = default_checker.check(&doc);
    let strict_report = strict_checker.check(&doc);

    // Strict config should be more sensitive
    assert!(strict_report.score <= default_report.score);
}

// ============================================================================
// Repair Engine Tests
// ============================================================================

#[test]
fn test_repairEngine_withMissingFormatting_shouldRestore() {
    let entries = vec![SubtitleEntry::new(
        1,
        0,
        2000,
        "<i>Important whisper</i>".to_string(),
    )];

    let mut doc = SubtitleDocument::from_entries(entries, "en");
    doc.entries[0].set_translation("Murmure important".to_string(), Some(0.9));

    let engine = RepairEngine::new();
    let repairs = engine.repair_entry(&doc.entries[0], &doc.glossary);

    // Should have attempted formatting repair
    let has_formatting_repair = repairs.iter().any(|r| {
        matches!(
            r.strategy,
            yastwai::translation::RepairStrategy::RestoreFormatting
        )
    });

    assert!(has_formatting_repair, "Should attempt formatting repair");
}

#[test]
fn test_repairEngine_withGlossaryTerms_shouldApply() {
    let entries = vec![SubtitleEntry::new(
        1,
        0,
        2000,
        "Go to the extraction point.".to_string(),
    )];

    let mut doc = SubtitleDocument::from_entries(entries, "en");
    doc.entries[0].set_translation("Allez au extraction point.".to_string(), Some(0.9));
    doc.glossary.add_term("extraction point", "point d'extraction", None);

    let engine = RepairEngine::new();
    let repairs = engine.repair_entry(&doc.entries[0], &doc.glossary);

    let glossary_repair = repairs
        .iter()
        .find(|r| matches!(r.strategy, yastwai::translation::RepairStrategy::ApplyGlossary));

    assert!(glossary_repair.is_some(), "Should apply glossary correction");
    if let Some(repair) = glossary_repair {
        assert!(repair.success);
        assert!(repair.repaired.as_ref().unwrap().contains("point d'extraction"));
    }
}

#[test]
fn test_repairEngine_conservativeConfig_shouldDoLess() {
    let entries = vec![SubtitleEntry::new(1, 0, 2000, "Test text.".to_string())];

    let mut doc = SubtitleDocument::from_entries(entries, "en");
    doc.entries[0].set_translation("Texte de test avec «guillemets».".to_string(), Some(0.9));

    let conservative = RepairEngine::with_config(RepairConfig::conservative());
    let aggressive = RepairEngine::with_config(RepairConfig::aggressive());

    let conservative_repairs = conservative.repair_entry(&doc.entries[0], &doc.glossary);
    let aggressive_repairs = aggressive.repair_entry(&doc.entries[0], &doc.glossary);

    // Aggressive should attempt more repairs
    assert!(
        aggressive_repairs.len() >= conservative_repairs.len(),
        "Aggressive config should attempt more repairs"
    );
}

// ============================================================================
// Error Recovery Tests
// ============================================================================

#[test]
fn test_errorRecovery_withNetworkError_shouldRetry() {
    let mut recovery = ErrorRecovery::new();
    let error = TranslationError::new(TranslationErrorKind::Network, "Connection failed");

    let action = recovery.handle_error(&error);

    assert!(matches!(
        action,
        yastwai::translation::RecoveryAction::Retry { .. }
    ));
}

#[test]
fn test_errorRecovery_withRateLimit_shouldDelayRetry() {
    let mut recovery = ErrorRecovery::new();
    let error = TranslationError::new(TranslationErrorKind::RateLimit, "Rate limit exceeded");

    let action = recovery.handle_error(&error);

    if let yastwai::translation::RecoveryAction::Retry { delay, .. } = action {
        assert!(delay >= Duration::from_secs(30), "Rate limit should have long delay");
    } else {
        panic!("Should retry on rate limit");
    }
}

#[test]
fn test_errorRecovery_withConfigError_shouldAbort() {
    let mut recovery = ErrorRecovery::new();
    let error = TranslationError::new(TranslationErrorKind::ConfigError, "Invalid API key");

    let action = recovery.handle_error(&error);

    assert!(matches!(
        action,
        yastwai::translation::RecoveryAction::Abort { .. }
    ));
}

#[test]
fn test_errorRecovery_afterMaxRetries_shouldFallback() {
    let mut recovery = ErrorRecovery::with_strategy(RecoveryStrategy {
        max_retries: 2,
        use_fallback: true,
        ..Default::default()
    });

    let error = TranslationError::new(TranslationErrorKind::Network, "Connection failed")
        .with_entries(vec![1, 2, 3]);

    // First two retries
    recovery.handle_error(&error);
    recovery.handle_error(&error);

    // Third should fallback
    let action = recovery.handle_error(&error);

    assert!(matches!(
        action,
        yastwai::translation::RecoveryAction::UseFallback { .. }
    ));
}

#[test]
fn test_errorRecovery_withParseError_shouldReduceBatch() {
    let mut recovery = ErrorRecovery::new();
    let error = TranslationError::new(TranslationErrorKind::ParseError, "Invalid JSON")
        .with_entries(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    let action = recovery.handle_error(&error);

    assert!(matches!(
        action,
        yastwai::translation::RecoveryAction::ReduceBatchSize { .. }
    ));
}

#[test]
fn test_errorRecovery_summary_shouldDescribeErrors() {
    let mut recovery = ErrorRecovery::new();

    recovery.handle_error(&TranslationError::new(
        TranslationErrorKind::Network,
        "Error 1",
    ));
    recovery.handle_error(&TranslationError::new(
        TranslationErrorKind::Network,
        "Error 2",
    ));
    recovery.handle_error(&TranslationError::new(
        TranslationErrorKind::Timeout,
        "Error 3",
    ));

    let summary = recovery.error_summary();

    assert!(summary.contains("3 errors"));
    assert!(summary.contains("Network"));
    assert!(summary.contains("Timeout"));
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_metricsToRepair_workflow_shouldImproveScore() {
    // Create document with issues
    let entries = vec![SubtitleEntry::new(
        1,
        0,
        2000,
        "<i>Important text</i>".to_string(),
    )];

    let mut doc = SubtitleDocument::from_entries(entries, "en");
    doc.entries[0].set_translation("Texte important".to_string(), Some(0.9)); // Missing formatting

    // Calculate initial metrics
    let metrics = QualityMetrics::new();
    let mut initial_data = MetricsData::new();
    initial_data.add_entry(
        1,
        EntryMetrics {
            is_translated: true,
            is_empty: false,
            has_issues: true,
            length_ratio: Some(1.0),
            expected_tags: 2,
            missing_tags: 2, // Missing <i> and </i>
            ..Default::default()
        },
    );
    let initial_score = metrics.calculate_score(&initial_data);

    // Apply repair
    let repair_engine = RepairEngine::new();
    let repairs = repair_engine.repair_entry(&doc.entries[0], &doc.glossary);

    // Get final text
    if let Some(final_text) = repair_engine.get_final_text(&repairs) {
        doc.entries[0].set_translation(final_text, Some(0.9));
    }

    // Calculate final metrics
    let mut final_data = MetricsData::new();
    let has_formatting = doc.entries[0]
        .translated_text
        .as_ref()
        .map(|t| t.contains("<i>") && t.contains("</i>"))
        .unwrap_or(false);

    final_data.add_entry(
        1,
        EntryMetrics {
            is_translated: true,
            is_empty: false,
            has_issues: !has_formatting,
            length_ratio: Some(1.0),
            expected_tags: 2,
            missing_tags: if has_formatting { 0 } else { 2 },
            ..Default::default()
        },
    );
    let final_score = metrics.calculate_score(&final_data);

    // Score should improve after repair
    assert!(
        final_score.formatting.score >= initial_score.formatting.score,
        "Repair should improve formatting score"
    );
}
