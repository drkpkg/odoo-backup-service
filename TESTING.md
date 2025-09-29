# Testing Documentation

This document describes the testing strategy and implementation for the Odoo Backup Service.

## Test Structure

The project uses a comprehensive testing approach with unit tests for each module:

### Unit Tests

All unit tests are located within each module's `#[cfg(test)]` section:

- **`src/config.rs`** - 15 tests covering configuration parsing and validation
- **`src/error.rs`** - 6 tests covering error handling and type conversions
- **`src/docker.rs`** - 10 tests covering Docker command construction and logic
- **`src/backup.rs`** - 11 tests covering backup management and file operations
- **`src/cli.rs`** - 16 tests covering CLI argument parsing and command handling

### Test Coverage

**Total: 58 unit tests** covering:

#### Configuration Module (`config.rs`)
- JSON file parsing (valid and invalid)
- Configuration validation (all required fields)
- Database lookup functionality
- Error handling for missing files and malformed JSON
- Backup format validation (zip/dump)
- Empty configuration handling

#### Error Handling Module (`error.rs`)
- Error display formatting
- Error type conversions (IO, JSON, HTTP)
- Result type alias functionality
- Error debug formatting

#### Docker Module (`docker.rs`)
- Docker manager creation
- Backup command construction
- Filename generation with timestamps
- Path construction (container and host)
- Docker command argument validation
- File copy and cleanup command construction

#### Backup Module (`backup.rs`)
- Backup manager creation and configuration
- Backup filename pattern generation
- Retention policy calculations
- File filtering logic
- Multiple database configuration handling
- Custom directory handling

#### CLI Module (`cli.rs`)
- All command parsing (backup, list, status, clean, list-backups)
- Command options and flags (short and long form)
- Default value handling
- Client-specific operations
- Help and version commands
- Verbose logging flag

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Unit Tests Only
```bash
cargo test --lib
```

### Run Tests with Output
```bash
cargo test -- --nocapture
```

### Run Specific Module Tests
```bash
cargo test config::tests
cargo test error::tests
cargo test docker::tests
cargo test backup::tests
cargo test cli::tests
```

### Run Tests in Release Mode
```bash
cargo test --release
```

## Test Dependencies

The following development dependencies are used for testing:

- **`tempfile`** - For creating temporary directories and files in tests
- **`tokio`** - For async test support
- **`serde_json`** - For JSON parsing in configuration tests

## Test Utilities

### Helper Functions

Each test module includes helper functions for creating test data:

- `create_test_config()` - Creates a standard test database configuration
- `create_test_configs()` - Creates multiple test configurations
- `create_test_database_config()` - Creates test database configs for Docker/backup tests
- `create_test_backup_manager()` - Creates a test backup manager instance

### Test Data

Tests use consistent, predictable test data:
- **Database Name**: `test_database`
- **Container Name**: `test_container`
- **URL**: `http://localhost:8069`
- **Master Password**: `admin`
- **Backup Format**: `zip`
- **Retention Days**: `30`

## Test Categories

### 1. Configuration Tests
- **File I/O**: Reading configuration files, handling missing files
- **Validation**: Required field validation, format validation
- **Parsing**: JSON parsing, error handling
- **Lookup**: Database finding and retrieval

### 2. Error Handling Tests
- **Type Conversion**: Converting between error types
- **Display**: Error message formatting
- **Debug**: Debug representation
- **Result Types**: Custom Result type functionality

### 3. Docker Integration Tests
- **Command Construction**: Building Docker commands correctly
- **Path Handling**: Container and host path construction
- **Filename Generation**: Timestamp-based backup filenames
- **Argument Validation**: Ensuring correct command arguments

### 4. Backup Management Tests
- **File Operations**: Backup file listing and filtering
- **Retention Policies**: Cleanup logic and date calculations
- **Configuration**: Multiple database handling
- **Directory Management**: Backup directory creation and management

### 5. CLI Interface Tests
- **Command Parsing**: All CLI commands and subcommands
- **Option Handling**: Short and long form options
- **Default Values**: Proper default value assignment
- **Error Cases**: Invalid command handling

## Test Quality Metrics

### Coverage Areas
- **Happy Path**: Normal operation scenarios
- **Error Cases**: Invalid input and failure scenarios
- **Edge Cases**: Boundary conditions and special cases
- **Integration Points**: Module interaction testing

### Test Reliability
- **Deterministic**: Tests produce consistent results
- **Isolated**: Tests don't depend on external state
- **Fast**: All tests run quickly (< 1 second)
- **Clear**: Test names and assertions are descriptive

## Future Testing Considerations

### Integration Tests
For full integration testing, consider adding:

1. **Docker Integration Tests**
   - Test with actual Docker containers
   - Mock Odoo API endpoints
   - Test backup file creation and transfer

2. **End-to-End Tests**
   - Complete backup workflow testing
   - Error recovery testing
   - Performance testing

3. **Mock Testing**
   - Mock Docker commands
   - Mock file system operations
   - Mock network requests

### Test Environment Setup
For comprehensive testing, you would need:

1. **Test Docker Environment**
   - Test Odoo containers
   - Mock Odoo API server
   - Test data setup

2. **Test Data Management**
   - Sample backup files
   - Test configuration files
   - Cleanup procedures

## Continuous Integration

The test suite is designed to run in CI environments:

- **No External Dependencies**: Tests don't require Docker or Odoo
- **Fast Execution**: Complete test suite runs in seconds
- **Deterministic**: Consistent results across environments
- **Comprehensive**: Covers all major functionality

## Test Maintenance

### Adding New Tests
1. Follow existing naming conventions (`test_*`)
2. Use helper functions for test data creation
3. Test both success and failure cases
4. Include descriptive assertions

### Updating Tests
1. Update tests when changing functionality
2. Maintain test data consistency
3. Ensure tests remain fast and reliable
4. Update documentation when adding new test categories

## Test Results

Current test status: **58 tests passing, 0 failing**

All tests pass consistently and provide comprehensive coverage of the application's functionality.
