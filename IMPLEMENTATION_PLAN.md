# Implementation Plan - Clprs

## Phase 1: Core Layout Correction (MVP)

### Architecture Overview
- Modular design with separate modules for clipboard, layout mapping, detection, and configuration
- Configuration-driven approach with default ru/en layout pair
- Error handling throughout the pipeline

### Implementation Steps

1. **Dependencies & Project Setup**
   - Add clipboard access crate (cross-platform)
   - Add configuration management (TOML/JSON)
   - Set up basic error handling framework

2. **Configuration System**
   - Default config: Russian ↔ English layout pair
   - Support for custom layout mappings
   - Configuration file location (~/.config/clprs/config.toml)

3. **Layout Mapping Module**
   - Russian-English character mapping tables
   - Bidirectional conversion functions
   - Preserve punctuation, spaces, and special characters

4. **Layout Detection Algorithm**
   - Statistical analysis of character frequencies
   - Dictionary-based word recognition
   - Heuristics for mixed-language text

5. **Clipboard Integration**
   - Cross-platform clipboard read/write
   - Permission handling
   - Text format validation

6. **Main CLI Interface**
   - Single command execution: `clprs`
   - Silent success, verbose errors
   - Optional flags for manual layout specification

7. **Error Handling & Edge Cases**
   - Clipboard access failures
   - Empty clipboard handling
   - Non-text content detection
   - Invalid character sequences

### Key Components
- `src/clipboard.rs` - Cross-platform clipboard operations
- `src/layout.rs` - Layout mapping and conversion logic
- `src/detection.rs` - Layout detection algorithms  
- `src/config.rs` - Configuration management
- `src/main.rs` - CLI interface and orchestration

### Performance Targets
- <100ms processing time
- Minimal memory footprint
- Zero-configuration default behavior

### Dependencies Required
- `clipboard` or `arboard` - Cross-platform clipboard access
- `serde` + `toml` - Configuration serialization
- `thiserror` - Error handling
- `dirs` - User directories