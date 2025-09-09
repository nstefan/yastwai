# YASTwAI Troubleshooting Guide

## Common Issues and Solutions

### Installation and Setup

#### FFmpeg Not Found
**Error:** `FFmpeg not found in PATH` or similar extraction errors

**Solution:**
1. Install FFmpeg on your system:
   - **macOS:** `brew install ffmpeg`
   - **Ubuntu/Debian:** `sudo apt install ffmpeg`
   - **Windows:** Download from https://ffmpeg.org/download.html
2. Ensure FFmpeg is in your system PATH
3. Test with: `ffmpeg -version`

#### Rust/Cargo Issues
**Error:** `cargo: command not found` or build failures

**Solution:**
1. Install Rust: https://rustup.rs/
2. Update Rust: `rustup update`
3. For build errors, try: `cargo clean && cargo build`

### Configuration Issues

#### Invalid Configuration File
**Error:** `Failed to parse config file` or JSON syntax errors

**Solution:**
1. Validate JSON syntax at https://jsonlint.com/
2. Check for missing commas, quotes, or brackets
3. Delete config file to regenerate: `rm conf.json`
4. Use `--log-level debug` for detailed error messages

**Example of common JSON errors:**
```json
// ❌ Incorrect (trailing comma, comments not allowed)
{
  "source_language": "en",
  "target_language": "es", // This comment breaks JSON
}

// ✅ Correct
{
  "source_language": "en",
  "target_language": "es"
}
```

#### Missing API Keys
**Error:** `Authentication failed` or `API key not found`

**Solution:**
1. Set environment variables:
   ```bash
   export OPENAI_API_KEY="your-key-here"
   export ANTHROPIC_API_KEY="your-key-here"
   ```
2. Verify with: `echo $OPENAI_API_KEY`
3. Check that `api_key_env` in config matches your environment variable name

#### Provider Connection Issues
**Error:** `Failed to connect to provider` or timeout errors

**Solution:**
1. **For Ollama:**
   - Check if Ollama is running: `ollama list`
   - Start Ollama: `ollama serve`
   - Verify model is available: `ollama pull llama3.2:3b`
   - Check endpoint URL in config (default: http://localhost:11434)

2. **For OpenAI/Anthropic:**
   - Verify internet connection
   - Check API key validity
   - Verify endpoint URLs are correct
   - Try increasing `timeout_seconds` in config

### Translation Issues

#### No Subtitles Found
**Error:** `No subtitle tracks found in file`

**Solution:**
1. Check if file actually contains subtitles: `ffmpeg -i movie.mkv`
2. For files without embedded subtitles, use external SRT files
3. Verify file format is supported (MP4, MKV, AVI, etc.)

#### Extraction Failures
**Error:** `Failed to extract subtitle` or FFmpeg errors

**Solution:**
1. Check file permissions (read access required)
2. Ensure sufficient disk space
3. Try a different subtitle track: `yastwai -e --extract-language en movie.mkv`
4. For corrupted files, try re-downloading or using a different source

#### Translation Quality Issues
**Problem:** Poor translation quality or incorrect language detection

**Solution:**
1. **Try a different model:**
   ```bash
   yastwai --provider openai --model gpt-4o movie.mkv
   ```

2. **Adjust temperature for consistency:**
   - Lower temperature (0.1-0.3) for more consistent output
   - Higher temperature (0.7-1.0) for more creative translations

3. **Specify source language explicitly:**
   ```bash
   yastwai --source-language ja --target-language en movie.mkv
   ```

4. **Use smaller chunks for complex content:**
   - Reduce `max_chars_per_request` in config to 2000-3000

#### Rate Limiting
**Error:** `Rate limit exceeded` or `Too many requests`

**Solution:**
1. Reduce `optimal_concurrent_requests` in config
2. Increase `retry_delay_ms` between attempts
3. For OpenAI: Check your tier limits at https://platform.openai.com/settings/organization/limits
4. For Anthropic: Start with lower concurrent requests (1-2)

### Performance Issues

#### Slow Translation Speed
**Problem:** Translation takes too long

**Solution:**
1. **Optimize configuration:**
   ```json
   {
     "max_chars_per_request": 6000,
     "optimal_concurrent_requests": 5,
     "timeout_seconds": 30
   }
   ```

2. **Use faster models:**
   - OpenAI: `gpt-4o-mini` instead of `gpt-4o`
   - Anthropic: `claude-3-5-haiku` instead of `claude-3-5-sonnet`

3. **Check system resources:**
   - Ensure sufficient RAM and CPU
   - Close other applications
   - Use SSD storage for better I/O

#### High Memory Usage
**Problem:** System runs out of memory

**Solution:**
1. Reduce `max_chars_per_request` to lower memory usage
2. Process files one at a time instead of batch processing
3. Use `--log-level error` to reduce memory from logging
4. For large files, split into smaller segments

### File Handling Issues

#### Permission Denied
**Error:** `Permission denied` when reading/writing files

**Solution:**
1. Check file permissions: `ls -la movie.mkv`
2. Ensure read access to input files
3. Ensure write access to output directory
4. On Windows, run as administrator if needed

#### Output File Already Exists
**Error:** `Output file already exists` (without force flag)

**Solution:**
1. Use `--force` flag to overwrite: `yastwai -f movie.mkv`
2. Or rename/move existing output files
3. Check if translation actually completed successfully

#### Unsupported File Format
**Error:** `Unsupported file format` or extraction failures

**Solution:**
1. **Supported video formats:** MP4, MKV, AVI, MOV, WMV, FLV, WebM
2. **For SRT files:** Use directly as input (no extraction needed)
3. **For other formats:** Convert using FFmpeg:
   ```bash
   ffmpeg -i input.format -c copy output.mkv
   ```

### Provider-Specific Issues

#### Ollama Issues

**Model Not Found:**
```bash
# List available models
ollama list

# Pull the required model
ollama pull llama3.2:3b

# Verify model works
ollama run llama3.2:3b "Hello"
```

**Ollama Not Responding:**
```bash
# Check if Ollama is running
ps aux | grep ollama

# Start Ollama
ollama serve

# Check logs
ollama logs
```

#### OpenAI Issues

**API Key Problems:**
1. Verify key at https://platform.openai.com/api-keys
2. Check account billing status
3. Ensure key has sufficient credits

**Model Access Issues:**
1. Some models require approval or higher tier
2. Check model availability in your region
3. Try `gpt-4o-mini` for wider availability

#### Anthropic Issues

**API Access:**
1. Ensure you have access to the Anthropic API
2. Check your console at https://console.anthropic.com/
3. Verify billing and usage limits

**Model Naming:**
- Use exact model names from Anthropic documentation
- Model names change over time, update config accordingly

### Debugging Tips

#### Enable Debug Logging
```bash
yastwai --log-level debug movie.mkv
```

#### Check Configuration
```bash
yastwai --help  # View all options
yastwai --config-path /path/to/config.json --log-level debug movie.mkv
```

#### Test Individual Components

**Test subtitle extraction only:**
```bash
yastwai -e movie.mkv
```

**Test with minimal config:**
```bash
yastwai --provider ollama --model llama3.2:3b --source-language en --target-language es movie.mkv
```

#### Examine Log Files
YASTwAI creates log files in the output directory:
- `yastwai.issues.log` - Translation warnings and errors
- Check timestamps to correlate with issues

### Getting Help

#### Reporting Issues

When reporting issues, include:
1. **Version:** `yastwai --version`
2. **Command used:** Full command line
3. **Configuration:** Sanitized config file (remove API keys)
4. **Error output:** Complete error messages
5. **Environment:** OS, Rust version, FFmpeg version
6. **File information:** File format, size, subtitle tracks

#### Log Information
Use debug logging for detailed information:
```bash
yastwai --log-level debug movie.mkv 2>&1 | tee debug.log
```

#### System Information
```bash
# Check versions
yastwai --version
ffmpeg -version
rustc --version

# Check system resources
df -h  # Disk space
free -h  # Memory (Linux)
top  # Running processes
```

### Recovery Procedures

#### Reset Configuration
```bash
# Backup current config
cp conf.json conf.json.backup

# Remove config to regenerate
rm conf.json

# Run yastwai to create new default config
yastwai --help
```

#### Clean Rebuild
```bash
# Clean build artifacts
cargo clean

# Rebuild from scratch
cargo build --release

# Test installation
./target/release/yastwai --version
```

#### Emergency Subtitle Extraction
If translation fails but you need the subtitles:
```bash
# Extract without translation
yastwai -e movie.mkv

# Manually translate the extracted SRT file
yastwai extracted.srt
```

### Prevention Tips

1. **Regular updates:** Keep YASTwAI, Rust, and FFmpeg updated
2. **Test configuration:** Validate config changes with simple files first
3. **Monitor resources:** Ensure sufficient disk space and memory
4. **Backup:** Keep backups of working configurations
5. **Documentation:** Document custom configurations and workflows

### Known Limitations

1. **Subtitle formats:** Only SRT output is currently supported
2. **Video formats:** Requires FFmpeg-compatible formats
3. **Language detection:** May struggle with mixed-language content
4. **Large files:** Memory usage scales with file size
5. **Network dependency:** Cloud providers require stable internet

For additional help, consult the [Configuration Guide](CONFIGURATION.md) and [Architecture Documentation](ARCHITECTURE.md).
