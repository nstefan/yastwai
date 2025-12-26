# Translation System Overhaul Plan

## Executive Summary

The current translation system uses fragile marker-based batching with limited context (3 previous entries). This plan proposes a complete architectural overhaul using **structured JSON communication**, **document-level context modeling**, and **multi-pass translation** for excellent quality and reliability.

---

## Part 1: Analysis of Current System Issues

### Current Architecture Problems

1. **Fragile Marker-Based Parsing**
   - Uses `<<ENTRY_X>>` and `[X]` markers that LLMs often break
   - Lots of retry/fallback logic due to marker parsing failures
   - 30+ lines of marker recovery code in `batch.rs`

2. **Limited Context Window**
   - Only 3 previous entries as context (`context_entries_count: 3`)
   - No understanding of overall narrative, characters, or terminology
   - Context passed as raw text, not structured

3. **Basic System Prompt**
   - Generic "You are a professional translator" prompt
   - No guidance on style, terminology consistency, or character voice

4. **Post-hoc Format Preservation**
   - Formatting applied after translation
   - Can break when translation structure differs from original

5. **No Document-Level Understanding**
   - Treats each batch independently
   - No scene detection or dialogue grouping
   - No character/speaker awareness

---

## Part 2: Proposed Architecture

### Core Principles

1. **JSON-Native Communication** - All LLM I/O uses structured JSON
2. **Document-First Model** - Build a rich document model before translation
3. **Multi-Pass Pipeline** - Analysis → Translation → Validation
4. **Sliding Context with Summaries** - Compress past context, keep recent detailed
5. **Terminology Enforcement** - Extract and maintain glossary consistency

### New Module Structure

```
src/translation/
├── mod.rs                    # Module exports
├── document/                 # Document modeling
│   ├── mod.rs
│   ├── model.rs              # SubtitleDocument, Scene, DialogueGroup
│   ├── analyzer.rs           # Scene detection, speaker inference
│   └── serialization.rs      # JSON serialization/deserialization
├── context/                  # Context management
│   ├── mod.rs
│   ├── window.rs             # Sliding window context
│   ├── summary.rs            # Context summarization
│   └── glossary.rs           # Terminology extraction & enforcement
├── pipeline/                 # Multi-pass pipeline
│   ├── mod.rs
│   ├── analysis_pass.rs      # Document analysis phase
│   ├── translation_pass.rs   # Main translation phase
│   ├── validation_pass.rs    # Quality validation phase
│   └── orchestrator.rs       # Pipeline coordination
├── prompts/                  # Prompt engineering
│   ├── mod.rs
│   ├── templates.rs          # Prompt templates
│   ├── builder.rs            # Dynamic prompt construction
│   └── strategies.rs         # Provider-specific strategies
└── quality/                  # Quality assurance
    ├── mod.rs
    ├── metrics.rs            # Translation quality metrics
    ├── consistency.rs        # Terminology/style consistency
    └── repair.rs             # Auto-repair strategies
```

---

## Part 3: Detailed Component Design

### 3.1 Document Model (`document/model.rs`)

Create a rich JSON-serializable model:

```rust
/// Complete subtitle document with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleDocument {
    /// Document metadata
    pub metadata: DocumentMetadata,
    /// Extracted characters/speakers (if detected)
    pub characters: Vec<Character>,
    /// Terminology glossary for consistency
    pub glossary: Glossary,
    /// Scene/chapter divisions
    pub scenes: Vec<Scene>,
    /// All subtitle entries
    pub entries: Vec<DocumentEntry>,
    /// Translation context summary
    pub context_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentEntry {
    pub id: usize,
    pub timecode: Timecode,
    pub original_text: String,
    pub translated_text: Option<String>,
    /// Detected speaker (if applicable)
    pub speaker: Option<String>,
    /// Scene this entry belongs to
    pub scene_id: Option<usize>,
    /// Formatting tags preserved
    pub formatting: Vec<FormattingTag>,
    /// Translation confidence score
    pub confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timecode {
    pub start_ms: u64,
    pub end_ms: u64,
    /// Original SRT format preserved
    pub original_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub id: usize,
    pub start_entry: usize,
    pub end_entry: usize,
    pub description: Option<String>,
    pub tone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glossary {
    /// Key terms with their translations
    pub terms: HashMap<String, GlossaryTerm>,
    /// Character names (never translate)
    pub character_names: HashSet<String>,
    /// Technical terms
    pub technical_terms: HashMap<String, String>,
}
```

### 3.2 Context Window (`context/window.rs`)

Implement sliding window with compressed history:

```rust
/// Smart context window for translation
pub struct ContextWindow {
    /// Summary of all content before the window
    pub history_summary: String,
    /// Recently translated entries (detailed)
    pub recent_entries: Vec<TranslatedEntry>,
    /// Current entries to translate
    pub current_batch: Vec<DocumentEntry>,
    /// Look-ahead entries (for context)
    pub lookahead_entries: Vec<DocumentEntry>,
    /// Active glossary terms
    pub active_glossary: Glossary,
}

impl ContextWindow {
    /// Window sizes (configurable)
    const RECENT_ENTRIES: usize = 10;  // Full recent context
    const LOOKAHEAD_ENTRIES: usize = 5; // Forward context
    const BATCH_SIZE: usize = 15;       // Entries per translation request
    
    /// Build context window for position in document
    pub fn build_for_position(doc: &SubtitleDocument, position: usize) -> Self;
    
    /// Generate summarized history using LLM
    pub async fn summarize_history(&mut self, service: &TranslationService) -> Result<()>;
    
    /// Convert to JSON for LLM prompt
    pub fn to_prompt_json(&self) -> Value;
}
```

### 3.3 Translation Pipeline (`pipeline/orchestrator.rs`)

Three-pass translation:

```rust
pub struct TranslationPipeline {
    service: TranslationService,
    config: PipelineConfig,
}

impl TranslationPipeline {
    /// Full document translation with quality assurance
    pub async fn translate_document(&self, doc: SubtitleDocument) -> Result<SubtitleDocument> {
        // Phase 1: Document Analysis
        let analyzed_doc = self.run_analysis_pass(doc).await?;
        
        // Phase 2: Main Translation with sliding window
        let translated_doc = self.run_translation_pass(analyzed_doc).await?;
        
        // Phase 3: Quality Validation & Repair
        let validated_doc = self.run_validation_pass(translated_doc).await?;
        
        Ok(validated_doc)
    }
}
```

#### Phase 1: Analysis Pass

```rust
pub struct AnalysisPass;

impl AnalysisPass {
    /// Analyze document for context extraction
    pub async fn analyze(&self, doc: &SubtitleDocument) -> Result<AnalysisResult> {
        // 1. Detect scenes based on timing gaps
        let scenes = self.detect_scenes(&doc.entries)?;
        
        // 2. Extract character names and terminology
        let glossary = self.extract_glossary(&doc, source_lang).await?;
        
        // 3. Infer tone and style
        let style_analysis = self.analyze_style(&doc).await?;
        
        // 4. Create document summary
        let summary = self.summarize_document(&doc).await?;
        
        Ok(AnalysisResult { scenes, glossary, style_analysis, summary })
    }
}
```

#### Phase 2: Translation Pass (JSON-Based)

```rust
pub struct TranslationPass;

impl TranslationPass {
    /// Translate with structured JSON I/O
    pub async fn translate_batch(
        &self,
        window: &ContextWindow,
        target_lang: &str,
    ) -> Result<Vec<TranslatedEntry>> {
        // Build structured prompt
        let request = TranslationRequest {
            task: "translate_subtitles",
            source_language: &window.source_lang,
            target_language: target_lang,
            context: ContextData {
                history_summary: &window.history_summary,
                recent_translations: &window.recent_entries,
                lookahead: &window.lookahead_entries,
                glossary: &window.active_glossary,
            },
            entries_to_translate: &window.current_batch,
            instructions: &self.build_instructions(),
        };
        
        // Request JSON response
        let response: TranslationResponse = self.service
            .complete_json(request)
            .await?;
        
        // Validate response structure matches request
        self.validate_response(&window.current_batch, &response)?;
        
        Ok(response.translations)
    }
}
```

**LLM JSON Request Format:**

```json
{
  "task": "translate_subtitles",
  "source_language": "English",
  "target_language": "French",
  "context": {
    "history_summary": "John and Mary are discussing their plan to escape the city...",
    "recent_translations": [
      {"id": 45, "original": "We need to go now.", "translated": "Nous devons partir maintenant."},
      {"id": 46, "original": "But what about Sarah?", "translated": "Mais qu'en est-il de Sarah ?"}
    ],
    "lookahead": [
      {"id": 52, "original": "The helicopter is waiting."},
      {"id": 53, "original": "We don't have much time."}
    ],
    "glossary": {
      "character_names": ["John", "Mary", "Sarah"],
      "terms": {"the facility": "l'établissement", "extraction point": "point d'extraction"}
    }
  },
  "entries_to_translate": [
    {"id": 47, "text": "She's at the extraction point.", "timecode": "00:05:23,100 --> 00:05:25,500"},
    {"id": 48, "text": "Then let's move!", "timecode": "00:05:25,800 --> 00:05:27,200"},
    {"id": 49, "text": "[Gunshots in distance]", "timecode": "00:05:27,500 --> 00:05:29,000"},
    {"id": 50, "text": "We need backup.", "timecode": "00:05:29,300 --> 00:05:31,100"},
    {"id": 51, "text": "I'll call command.", "timecode": "00:05:31,400 --> 00:05:33,200"}
  ],
  "instructions": {
    "preserve_formatting": true,
    "maintain_tone": "tense, action",
    "match_length_ratio": 1.2,
    "preserve_sound_effects": true
  }
}
```

**LLM JSON Response Format:**

```json
{
  "translations": [
    {"id": 47, "translated": "Elle est au point d'extraction.", "confidence": 0.95},
    {"id": 48, "translated": "Alors allons-y !", "confidence": 0.98},
    {"id": 49, "translated": "[Coups de feu au loin]", "confidence": 0.92},
    {"id": 50, "translated": "On a besoin de renfort.", "confidence": 0.96},
    {"id": 51, "translated": "Je vais appeler le commandement.", "confidence": 0.94}
  ],
  "notes": {
    "glossary_updates": {"backup": "renfort"},
    "scene_context": "Action sequence, urgent tone maintained"
  }
}
```

### 3.4 Quality Validation (`quality/`)

```rust
pub struct ValidationPass;

impl ValidationPass {
    /// Validate and repair translations
    pub async fn validate(&self, doc: &SubtitleDocument) -> Result<ValidationReport> {
        let mut issues = Vec::new();
        
        // 1. Check all entries are translated
        issues.extend(self.check_completeness(doc));
        
        // 2. Check timing preservation
        issues.extend(self.check_timecodes(doc));
        
        // 3. Check formatting preservation
        issues.extend(self.check_formatting(doc));
        
        // 4. Check length ratios
        issues.extend(self.check_length_ratios(doc));
        
        // 5. Check glossary consistency
        issues.extend(self.check_glossary_consistency(doc));
        
        // 6. Attempt auto-repair for fixable issues
        let repaired_doc = self.auto_repair(doc, &issues).await?;
        
        Ok(ValidationReport { issues, repaired_doc })
    }
}
```

### 3.5 Provider-Specific JSON Mode

```rust
/// JSON completion for different providers
impl TranslationService {
    pub async fn complete_json<T: DeserializeOwned, R: Serialize>(
        &self,
        request: R,
    ) -> Result<T> {
        match &self.provider {
            TranslationProviderImpl::OpenAI { client } => {
                // Use JSON mode: response_format: { "type": "json_object" }
                client.complete_json(request).await
            },
            TranslationProviderImpl::Anthropic { client } => {
                // Use tool_use for structured output
                client.complete_with_tool(request).await
            },
            TranslationProviderImpl::Ollama { client } => {
                // Use format: "json" parameter
                client.generate_json(request).await
            },
            // ...
        }
    }
}
```

---

## Part 4: Prompt Engineering Strategy

### System Prompt Template

```
You are an expert subtitle translator specializing in {source_lang} to {target_lang} translation.

## Your Role
- Translate dialogue naturally while preserving meaning and emotion
- Maintain consistency with provided terminology/glossary
- Preserve formatting tags, sound effects, and speaker indicators
- Keep translations concise (subtitles have limited display time)

## Context Understanding
- Review the history summary to understand the narrative
- Reference recent translations for consistency
- Use lookahead entries to anticipate context
- Follow the glossary strictly for names and terms

## Output Requirements
- Return valid JSON matching the requested schema
- Include confidence scores (0.0-1.0) for each translation
- Flag any ambiguous translations in notes
- Suggest glossary updates for new recurring terms

## Quality Standards
- Natural, idiomatic {target_lang}
- Appropriate register (formal/informal) based on context
- Length within 120% of original where possible
- Preserve [sound effects] and (parentheticals) formatting
```

---

## Part 5: Implementation Roadmap

### Phase 1: Foundation (Week 1-2) ✅ COMPLETED
- [x] Create `SubtitleDocument` model and JSON serialization
- [x] Implement `Timecode` preservation (never modify timing)
- [x] Add JSON mode support to all providers
- [x] Create prompt template system

### Phase 2: Context System (Week 2-3) ✅ COMPLETED
- [x] Implement `ContextWindow` with sliding window
- [x] Add history summarization capability
- [x] Create glossary extraction and enforcement
- [x] Implement scene detection based on timing gaps

### Phase 3: Translation Pipeline (Week 3-4) ✅ COMPLETED
- [x] Build analysis pass (character/term extraction)
- [x] Implement main translation pass with JSON I/O
- [x] Add validation pass with quality metrics
- [x] Create pipeline orchestrator

### Phase 4: Quality & Reliability (Week 4-5)
- [ ] Implement length ratio validation
- [ ] Add consistency checking
- [ ] Create auto-repair strategies
- [ ] Add comprehensive error handling

### Phase 5: Testing & Optimization (Week 5-6)
- [ ] Unit tests for all new modules
- [ ] Integration tests with real SRT files
- [ ] Performance benchmarking
- [ ] Provider-specific optimizations

---

## Part 6: Key Technical Decisions

### 1. JSON Mode Over Markers

**Before (fragile):**
```
<<ENTRY_0>>
Translation here
<<ENTRY_1>>
Translation here
<<END>>
```

**After (reliable):**
```json
{
  "translations": [
    {"id": 0, "text": "Translation here"},
    {"id": 1, "text": "Translation here"}
  ]
}
```

### 2. Larger Batches with Context

**Before:** 3 entries as context, 3 entries per request  
**After:** Summary + 10 recent + 15 to translate + 5 lookahead = ~30 entries context per request

### 3. Timecode Immutability

```rust
impl DocumentEntry {
    /// Timecodes are NEVER modified during translation
    pub fn set_translated_text(&mut self, text: String) {
        self.translated_text = Some(text);
        // timecode remains unchanged
    }
}
```

### 4. Glossary-Driven Consistency

```rust
impl Glossary {
    /// Enforce terminology in translation
    pub fn enforce(&self, translation: &str) -> String {
        let mut result = translation.to_string();
        for (term, translation_term) in &self.terms {
            // Ensure consistent translation of key terms
            result = result.replace(term, &translation_term.target);
        }
        result
    }
}
```

---

## Part 7: Expected Improvements

| Metric | Current | Expected |
|--------|---------|----------|
| Marker parsing failures | ~5-10% | 0% (JSON mode) |
| Context entries | 3 | 10+ (recent) + summary |
| Terminology consistency | Low | High (glossary enforcement) |
| Retry rate | High | Low |
| Translation quality | Good | Excellent |
| Timing errors | Possible | Impossible (immutable) |

---

## Part 8: Configuration Additions

```json
{
  "translation": {
    "pipeline": {
      "enable_analysis_pass": true,
      "enable_validation_pass": true,
      "context_window": {
        "recent_entries": 10,
        "lookahead_entries": 5,
        "batch_size": 15,
        "enable_summarization": true
      },
      "glossary": {
        "auto_extract": true,
        "custom_terms": {}
      }
    }
  }
}
```

---

This plan transforms the translation system from a fragile marker-based approach to a robust, context-aware, JSON-native pipeline that will deliver significantly higher quality translations while maintaining perfect timing and format integrity.

