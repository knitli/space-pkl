#!/usr/bin/env bash

# PKL Test Runner for space-pkl
# Runs all PKL tests and generates reports

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEST_DIR="$PROJECT_ROOT/tests/pkl"
REPORTS_DIR="$PROJECT_ROOT/target/pkl-test-reports"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
  echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
  echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
  echo -e "${RED}[ERROR]${NC} $1"
}

# Check if pkl is available
check_pkl_availability() {
  if ! command -v pkl &>/dev/null; then
    log_error "PKL CLI is not available. Please install PKL first."
    exit 1
  fi

  local pkl_version
  pkl_version=$(pkl --version)
  log_info "Using $pkl_version"
}

# Create reports directory
setup_reports_dir() {
  mkdir -p "$REPORTS_DIR"
  log_info "Test reports will be saved to: $REPORTS_DIR"
}

# Run tests in a specific directory
run_test_category() {
  local category="$1"
  local test_path="$TEST_DIR/$category"

  if [[ ! -d "$test_path" ]]; then
    log_warn "No tests found in category: $category"
    return 0
  fi

  log_info "Running $category tests..."

  local test_files
  test_files=$(find "$test_path" -name "*.pkl" -type f)

  if [[ -z "$test_files" ]]; then
    log_warn "No .pkl test files found in $category"
    return 0
  fi

  local failed_tests=0
  local total_tests=0

  while IFS= read -r test_file; do
    ((total_tests++))
    local test_name
    test_name=$(basename "$test_file" .pkl)

    log_info "  Running test: $test_name"

    # Run pkl test and capture output
    if pkl test "$test_file" --junit-reports "$REPORTS_DIR" >"$REPORTS_DIR/${category}_${test_name}.log" 2>&1; then
      log_info "    ✓ PASSED: $test_name"
    else
      ((failed_tests++))
      log_error "    ✗ FAILED: $test_name"
      log_error "      Check $REPORTS_DIR/${category}_${test_name}.log for details"
    fi
  done <<<"$test_files"

  log_info "$category tests completed: $((total_tests - failed_tests))/$total_tests passed"
  return $failed_tests
}

# Run all PKL tests
run_all_tests() {
  local total_failures=0

  # Test categories to run
  local categories=(
    "schema_validation"
    "type_tests"
    "examples"
    "integration"
  )

  for category in "${categories[@]}"; do
    run_test_category "$category"
    total_failures=$((total_failures + $?))
  done

  return $total_failures
}

# Main execution
main() {
  log_info "Starting PKL tests for space-pkl..."

  check_pkl_availability
  setup_reports_dir

  # Change to project root for relative imports to work
  cd "$PROJECT_ROOT"

  if run_all_tests; then
    log_info "All PKL tests passed! ✓"
    exit 0
  else
    log_error "Some PKL tests failed! ✗"
    log_error "Check test reports in: $REPORTS_DIR"
    exit 1
  fi
}

main "$@"
