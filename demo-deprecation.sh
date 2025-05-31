#!/bin/bash
# Deprecation Feature Demonstration Script
# This script demonstrates the deprecation support in space-pkl

set -e

echo "🌙 space-pkl Deprecation Feature Demonstration"
echo "=============================================="
echo

# Clean up any existing output directories
rm -rf demo-output-*

echo "1. Generating schemas WITHOUT deprecated fields (default behavior)"
echo "================================================================="
./target/debug/space-pkl generate project -o demo-output-without-deprecated
echo "✅ Generated project schema without deprecated fields"
echo

echo "2. Generating schemas WITH deprecated fields included"
echo "==================================================="
./target/debug/space-pkl generate project --include-deprecated -o demo-output-with-deprecated
echo "✅ Generated project schema with deprecated fields"
echo

echo "3. Comparing the outputs to show differences"
echo "==========================================="
echo "Files without deprecated fields:"
wc -l demo-output-without-deprecated/project.pkl
echo "Files with deprecated fields:"
wc -l demo-output-with-deprecated/project.pkl
echo

echo "4. Differences in generated PKL schemas:"
echo "======================================="
if diff -u demo-output-without-deprecated/project.pkl demo-output-with-deprecated/project.pkl | head -20; then
    echo "No differences found in this section (deprecated fields might be elsewhere)"
else
    echo "✅ Found differences between schemas with and without deprecated fields"
fi
echo

echo "5. Running deprecation unit tests"
echo "================================="
cargo test --test deprecation_test --quiet
echo "✅ All deprecation tests passed"
echo

echo "6. Demonstration of CLI flags"
echo "============================"
echo "Available CLI options:"
./target/debug/space-pkl generate --help | grep -A5 -B5 deprecated
echo

echo "7. Summary of Implementation"
echo "============================"
echo "✅ Added 'deprecated: Option<String>' field to PklProperty struct"
echo "✅ Added 'deprecated: Option<String>' field to PklType struct for class-level deprecation"
echo "✅ Added 'include_deprecated: bool' field to GeneratorConfig (defaults to false)"
echo "✅ Added '--include-deprecated' CLI flag"
echo "✅ Updated property conversion to extract deprecation from schematic schemas"
echo "✅ Updated type conversion to handle class-level deprecation and filter deprecated properties"
echo "✅ Added template support for @Deprecated decorator rendering"
echo "✅ Added handlebars helper for deprecation decorator generation"
echo "✅ Comprehensive test suite covering all deprecation scenarios"
echo

echo "🎉 Deprecation support implementation is complete and fully functional!"
echo "   Use --include-deprecated flag to include deprecated fields in generated schemas"
