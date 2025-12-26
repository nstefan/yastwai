/*!
 * Translation pipeline for multi-pass subtitle translation.
 *
 * The pipeline processes documents through three phases:
 * 1. **Analysis Pass**: Extract characters, terminology, detect scenes, summarize
 * 2. **Translation Pass**: Translate entries with rich context using JSON I/O
 * 3. **Validation Pass**: Check quality, consistency, and auto-repair issues
 *
 * This architecture replaces the fragile marker-based approach with robust
 * JSON-native communication for reliable, high-quality translations.
 */

pub mod analysis_pass;
pub mod orchestrator;
pub mod translation_pass;
pub mod validation_pass;

// Re-export main types
pub use analysis_pass::{AnalysisPass, AnalysisResult};
pub use orchestrator::{PipelineConfig, PipelineProgress, TranslationPipeline};
pub use translation_pass::{TranslationPass, TranslationPassConfig};
pub use validation_pass::{
    RepairAction, RepairResult, ValidationIssue, ValidationPass, ValidationReport,
};
