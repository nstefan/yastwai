# YASTwAI Architecture Documentation

## Overview

YASTwAI (Yet Another Subtitle Translation with AI) is designed as a robust, modular subtitle translation tool that leverages multiple AI providers to translate subtitles extracted from video files. The architecture emphasizes async efficiency, modularity, and maintainability.

## Core Design Principles

### 1. **Modular Provider Architecture**
The system uses a trait-based approach for AI providers, allowing easy extension with new services:
- `Provider` trait defines the interface for all AI services
- Each provider (Ollama, OpenAI, Anthropic) implements specific client logic
- Configuration-driven provider selection
- Rate limiting and error handling per provider

### 2. **Async-First Design**
Built on Tokio for optimal I/O performance:
- Non-blocking file operations
- Concurrent translation processing
- Efficient batch operations
- Proper resource management with async patterns

### 3. **Separation of Concerns**
Clear module boundaries with distinct responsibilities:
```
src/
├── main.rs              # CLI entry point and argument handling
├── app_config.rs        # Configuration management
├── app_controller.rs    # Main workflow orchestration
├── subtitle_processor.rs # SRT parsing and subtitle extraction
├── file_utils.rs        # File system operations
├── language_utils.rs    # Language code validation and utilities
├── providers/           # AI provider implementations
│   ├── mod.rs          # Provider trait and common types
│   ├── ollama.rs       # Ollama provider implementation
│   ├── openai.rs       # OpenAI provider implementation
│   └── anthropic.rs    # Anthropic provider implementation
└── translation/        # Translation service and batching
    ├── mod.rs          # Translation service orchestration
    ├── core.rs         # Core translation logic
    ├── batch.rs        # Batch processing
    ├── cache.rs        # Translation caching
    └── formatting.rs   # Output formatting
```

## Key Components

### Configuration System (`app_config.rs`)
- **Purpose**: Centralized configuration management
- **Features**:
  - JSON-based configuration files
  - Environment variable support via CLI
  - Provider-specific settings
  - Validation and default values
- **Design**: Serde-based serialization with validation methods

### Subtitle Processing (`subtitle_processor.rs`)
- **Purpose**: Extract and parse subtitle data from video files
- **Features**:
  - FFmpeg integration for subtitle extraction
  - SRT format parsing and generation
  - Multi-track support with language detection
  - Auto-selection of appropriate subtitle tracks
- **Design**: Collection-based API with async operations

### Translation Service (`translation/`)
- **Purpose**: Orchestrate translation workflows
- **Features**:
  - Batch processing for efficiency
  - Configurable chunk sizes
  - Token usage tracking
  - Error recovery and retry logic
  - Progress reporting
- **Design**: Service + batch translator pattern

### Provider System (`providers/`)
- **Purpose**: Abstract AI provider implementations
- **Features**:
  - Unified Provider trait interface
  - Provider-specific optimizations
  - Rate limiting and backoff
  - Error categorization
- **Design**: Trait-based polymorphism with async support

## Data Flow

### 1. **Initialization Phase**
```
CLI Parsing → Configuration Loading → Provider Selection → Controller Setup
```

### 2. **Single File Translation**
```
Input File → File Type Detection → Subtitle Extraction → Translation Batching → AI Processing → Output Generation
```

### 3. **Folder Processing**
```
Directory Scan → File Filtering → Parallel Processing → Progress Tracking → Aggregated Reporting
```

## Async Patterns

### 1. **Concurrent Processing**
- Uses `tokio::spawn` for independent tasks
- `MultiProgress` for real-time feedback
- Bounded channels for backpressure management

### 2. **Resource Management**
- Connection pooling for HTTP clients
- Proper cleanup with RAII patterns
- Timeout handling for long-running operations

### 3. **Error Handling**
- Structured error types with `thiserror`
- Graceful degradation on partial failures
- Comprehensive logging with contextual information

## Performance Considerations

### 1. **Batching Strategy**
- Dynamic chunk sizing based on provider limits
- Character-based segmentation (not just subtitle count)
- Parallel batch processing within rate limits

### 2. **Memory Management**
- Streaming where possible
- Temporary file cleanup
- Bounded buffer sizes

### 3. **I/O Optimization**
- Async file operations
- Minimal copying between layers
- Efficient progress reporting

## Testing Architecture

### 1. **Unit Tests**
- Mock providers for isolated testing
- Property-based testing for parsers
- Error scenario coverage

### 2. **Integration Tests**
- End-to-end workflow validation
- Real provider testing (opt-in)
- Performance benchmarking

### 3. **Test Organization**
```
tests/
├── unit/              # Unit tests per module
├── integration/       # End-to-end scenarios
├── common/           # Shared test utilities
├── resources/        # Test data files
└── scripts/          # Script testing
```

## Extension Points

### 1. **Adding New Providers**
1. Implement the `Provider` trait
2. Add provider-specific configuration
3. Register in the provider factory
4. Add tests and documentation

### 2. **Custom Processing Pipelines**
- Extend `SubtitleProcessor` for new formats
- Add middleware for custom transformations
- Implement custom batch strategies

### 3. **Configuration Extensions**
- Add new configuration sections
- Implement validation logic
- Update CLI argument parsing

## Security Considerations

### 1. **API Key Management**
- Environment variable support
- Configuration file protection
- No key logging or exposure

### 2. **Input Validation**
- File type verification
- Path traversal prevention
- Content sanitization

### 3. **Network Security**
- HTTPS enforcement
- Certificate validation
- Request signing where supported

## Future Architecture Considerations

### 1. **Scalability**
- Microservice decomposition potential
- Database backend for large-scale operations
- Distributed processing capabilities

### 2. **Observability**
- Structured logging enhancement
- Metrics collection
- Distributed tracing integration

### 3. **Configuration**
- Hot configuration reloading
- Advanced validation schemas
- Environment-specific configurations

## Technology Stack

- **Language**: Rust (Edition 2024)
- **Async Runtime**: Tokio
- **HTTP Client**: Reqwest
- **CLI Framework**: Clap v4
- **Configuration**: Serde JSON
- **Error Handling**: Anyhow + Thiserror
- **Testing**: Built-in test framework + Tokio-test
- **External Tools**: FFmpeg for subtitle extraction

## Design Patterns

### 1. **Builder Pattern**
Used for request construction in providers

### 2. **Strategy Pattern**
Provider selection and batch processing strategies

### 3. **Observer Pattern**
Progress reporting and logging

### 4. **Factory Pattern**
Provider instantiation and configuration

## Performance Metrics

### 1. **Translation Speed**
- Typical: 100-500 subtitles/minute (provider dependent)
- Batch optimization: 2-5x improvement over sequential

### 2. **Memory Usage**
- Base: ~10MB
- Per file: ~1-5MB additional
- Bounded by chunk sizes

### 3. **Error Rates**
- Target: <1% for well-formed inputs
- Graceful degradation on provider failures
- Comprehensive retry mechanisms

This architecture provides a solid foundation for reliable subtitle translation while maintaining flexibility for future enhancements and provider integrations.
