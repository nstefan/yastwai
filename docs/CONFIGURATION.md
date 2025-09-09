# YASTwAI Configuration Guide

## Overview

YASTwAI uses a JSON-based configuration system that allows you to customize translation providers, languages, performance settings, and more. Configuration files provide a convenient way to set defaults while CLI arguments allow for per-run overrides.

## Configuration File Location

By default, YASTwAI looks for a configuration file named `conf.json` in the current directory. You can specify a different location using the `--config-path` CLI option:

```bash
yastwai --config-path /path/to/my-config.json movie.mkv
```

If no configuration file exists, YASTwAI will automatically create a default one.

## Complete Configuration Schema

```json
{
  "source_language": "en",
  "target_language": "es",
  "log_level": "info",
  "translation": {
    "provider": "ollama",
    "max_chars_per_request": 4000,
    "optimal_concurrent_requests": 3,
    "max_retries": 3,
    "retry_delay_ms": 1000,
    "timeout_seconds": 30,
    "available_providers": [
      {
        "provider_type": "ollama",
        "endpoint": "http://localhost:11434",
        "model": "llama3.2:3b",
        "temperature": 0.7
      },
      {
        "provider_type": "openai",
        "endpoint": "https://api.openai.com/v1",
        "model": "gpt-4o-mini",
        "temperature": 0.7,
        "api_key_env": "OPENAI_API_KEY"
      },
      {
        "provider_type": "anthropic",
        "endpoint": "https://api.anthropic.com",
        "model": "claude-3-5-haiku-20241022",
        "temperature": 0.7,
        "api_key_env": "ANTHROPIC_API_KEY"
      }
    ]
  }
}
```

## Configuration Sections

### Global Settings

#### `source_language` (string, required)
The default source language for translation. Should be a 2-3 letter language code.

**Examples:**
- `"en"` - English
- `"es"` - Spanish  
- `"fr"` - French
- `"de"` - German
- `"ja"` - Japanese

#### `target_language` (string, required)
The default target language for translation.

**Examples:** Same format as `source_language`

#### `log_level` (string, optional)
Controls the verbosity of logging output.

**Valid values:**
- `"error"` - Only show errors
- `"warn"` - Show warnings and errors
- `"info"` - Show informational messages (default)
- `"debug"` - Show debug information
- `"trace"` - Show all internal details

### Translation Configuration

The `translation` section controls how translations are performed and which AI providers are used.

#### `provider` (string, required)
The default AI provider to use for translations.

**Valid values:**
- `"ollama"` - Local Ollama server
- `"openai"` - OpenAI API
- `"anthropic"` - Anthropic Claude API

#### `max_chars_per_request` (integer, optional)
Maximum number of characters to send in a single translation request. Larger values are more efficient but may hit provider limits.

**Default:** `4000`
**Range:** `100` - `32000`

#### `optimal_concurrent_requests` (integer, optional)
Number of translation requests to run concurrently. Higher values speed up processing but may hit rate limits.

**Default:** `3`
**Range:** `1` - `10`

#### `max_retries` (integer, optional)
Maximum number of retry attempts for failed translation requests.

**Default:** `3`
**Range:** `0` - `10`

#### `retry_delay_ms` (integer, optional)
Delay in milliseconds between retry attempts.

**Default:** `1000`
**Range:** `100` - `60000`

#### `timeout_seconds` (integer, optional)
Timeout for individual translation requests in seconds.

**Default:** `30`
**Range:** `5` - `300`

### Provider Configurations

The `available_providers` array contains configuration for each AI provider. Each provider configuration includes:

#### Common Provider Fields

##### `provider_type` (string, required)
Identifies the provider type. Must match one of the supported providers.

##### `endpoint` (string, required)
The API endpoint URL for the provider.

##### `model` (string, required)
The specific model to use with this provider.

##### `temperature` (float, optional)
Controls randomness in AI responses. Lower values are more deterministic.

**Default:** `0.7`
**Range:** `0.0` - `2.0`

#### Provider-Specific Fields

##### Ollama Provider
```json
{
  "provider_type": "ollama",
  "endpoint": "http://localhost:11434",
  "model": "llama3.2:3b",
  "temperature": 0.7
}
```

**Additional options:**
- Custom endpoint if running Ollama on different host/port

##### OpenAI Provider
```json
{
  "provider_type": "openai",
  "endpoint": "https://api.openai.com/v1",
  "model": "gpt-4o-mini",
  "temperature": 0.7,
  "api_key_env": "OPENAI_API_KEY"
}
```

**Additional options:**
- `api_key_env`: Environment variable name containing the API key

**Recommended models:**
- `gpt-4o-mini` - Cost-effective, good quality
- `gpt-4o` - Higher quality, more expensive
- `gpt-3.5-turbo` - Fast and economical

##### Anthropic Provider
```json
{
  "provider_type": "anthropic",
  "endpoint": "https://api.anthropic.com",
  "model": "claude-3-5-haiku-20241022",
  "temperature": 0.7,
  "api_key_env": "ANTHROPIC_API_KEY"
}
```

**Additional options:**
- `api_key_env`: Environment variable name containing the API key

**Recommended models:**
- `claude-3-5-haiku-20241022` - Fast and economical
- `claude-3-5-sonnet-20241022` - Balanced performance
- `claude-3-opus-20240229` - Highest quality

## Environment Variables

### API Keys

For cloud providers, API keys should be stored in environment variables:

```bash
# For OpenAI
export OPENAI_API_KEY="your-openai-api-key-here"

# For Anthropic
export ANTHROPIC_API_KEY="your-anthropic-api-key-here"
```

### CLI Overrides

All configuration values can be overridden via command-line arguments:

```bash
# Override provider and model
yastwai --provider openai --model gpt-4o movie.mkv

# Override languages
yastwai --source-language fr --target-language en movie.mkv

# Override log level
yastwai --log-level debug movie.mkv
```

## Configuration Examples

### Minimal Configuration
```json
{
  "source_language": "en",
  "target_language": "es",
  "translation": {
    "provider": "ollama",
    "available_providers": [
      {
        "provider_type": "ollama",
        "endpoint": "http://localhost:11434",
        "model": "llama3.2:3b"
      }
    ]
  }
}
```

### Production Configuration
```json
{
  "source_language": "en",
  "target_language": "es",
  "log_level": "info",
  "translation": {
    "provider": "openai",
    "max_chars_per_request": 6000,
    "optimal_concurrent_requests": 5,
    "max_retries": 3,
    "retry_delay_ms": 2000,
    "timeout_seconds": 60,
    "available_providers": [
      {
        "provider_type": "openai",
        "endpoint": "https://api.openai.com/v1",
        "model": "gpt-4o-mini",
        "temperature": 0.3,
        "api_key_env": "OPENAI_API_KEY"
      },
      {
        "provider_type": "anthropic",
        "endpoint": "https://api.anthropic.com",
        "model": "claude-3-5-haiku-20241022",
        "temperature": 0.3,
        "api_key_env": "ANTHROPIC_API_KEY"
      }
    ]
  }
}
```

### Development Configuration
```json
{
  "source_language": "en",
  "target_language": "es",
  "log_level": "debug",
  "translation": {
    "provider": "ollama",
    "max_chars_per_request": 2000,
    "optimal_concurrent_requests": 1,
    "max_retries": 1,
    "timeout_seconds": 120,
    "available_providers": [
      {
        "provider_type": "ollama",
        "endpoint": "http://localhost:11434",
        "model": "llama3.2:3b",
        "temperature": 0.1
      }
    ]
  }
}
```

## Performance Tuning

### For Speed
- Increase `max_chars_per_request` (up to provider limits)
- Increase `optimal_concurrent_requests` (watch for rate limits)
- Use faster models (e.g., `gpt-4o-mini`, `claude-3-5-haiku`)
- Reduce `timeout_seconds` for faster failure detection

### For Quality
- Use higher-quality models (e.g., `gpt-4o`, `claude-3-5-sonnet`)
- Lower `temperature` values (0.1-0.3) for more consistent output
- Decrease `max_chars_per_request` for more focused translations
- Increase `timeout_seconds` for complex translations

### For Cost Optimization
- Use cost-effective models (`gpt-4o-mini`, `claude-3-5-haiku`)
- Optimize `max_chars_per_request` to minimize API calls
- Use Ollama for local processing when possible
- Monitor token usage in logs

## Provider-Specific Considerations

### Ollama
- **Pros:** Free, private, no API keys needed
- **Cons:** Requires local setup, slower than cloud APIs
- **Setup:** Install Ollama and pull desired models
- **Best for:** Privacy-conscious users, development, experimentation

### OpenAI
- **Pros:** Fast, high quality, reliable
- **Cons:** Costs per token, requires internet
- **Rate limits:** Tier-dependent, typically generous
- **Best for:** Production use, high-quality translations

### Anthropic
- **Pros:** High quality, good at following instructions
- **Cons:** Costs per token, newer service
- **Rate limits:** Conservative initially, increases with usage
- **Best for:** High-quality translations, complex content

## Troubleshooting Configuration

### Common Issues

1. **Invalid JSON syntax**
   - Use a JSON validator to check syntax
   - Ensure proper quoting and comma placement

2. **Missing API keys**
   - Verify environment variables are set
   - Check `api_key_env` field matches your environment variable name

3. **Connection failures**
   - Verify endpoint URLs are correct
   - Check network connectivity and firewall settings

4. **Rate limit errors**
   - Reduce `optimal_concurrent_requests`
   - Increase `retry_delay_ms`

5. **Timeout errors**
   - Increase `timeout_seconds`
   - Reduce `max_chars_per_request`

### Validation

YASTwAI validates configuration on startup and provides helpful error messages for common issues. Always check the logs for specific validation failures.

### Configuration Migration

When updating YASTwAI, your existing configuration files will continue to work. New features may require adding new configuration sections, which will use sensible defaults if not specified.
