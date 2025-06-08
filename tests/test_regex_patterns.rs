//! Simple standalone test for the improved Rust doc link regex patterns
//! Run with: rustc test_regex_patterns.rs && ./test_regex_patterns

use regex::Regex;

fn main() {
    println!("Testing improved Rust doc link regex patterns...\n");

    // Define the improved regex patterns
    let backtick_regex = Regex::new(r"\[`(?P<ref>[^`\]]+)`\]").unwrap();
    let simple_regex = Regex::new(r"\[(?P<ref>[^\]`\(\)]+)\](?!\(|\[)").unwrap();
    let link_backticks_regex = Regex::new(r"\[(?P<text>[^\]]+)\]\(`(?P<ref>[^`\)]+)`\)").unwrap();
    let link_no_backticks_regex = Regex::new(r"\[(?P<text>[^\]]+)\]\((?P<ref>[^\)`]+)\)").unwrap();
    let reference_style_regex = Regex::new(r"\[(?P<text>[^\]]+)\]\[(?P<ref>[^\]]+)\]").unwrap();
    let reference_def_regex = Regex::new(r"^\s*\[(?P<ref>[^\]]+)\]:\s*(?P<target>.+)$").unwrap();

    // Test cases from the Rust documentation
    let test_cases = vec![
        // Basic patterns from Rust docs
        ("This struct is not [Bar]", "simple_regex", "Bar"),
        ("This struct is also not [bar](Bar)", "link_no_backticks_regex", "bar -> Bar"),
        ("This struct is also not [bar][b]", "reference_style_regex", "bar -> b"),
        ("This struct is also not [`Bar`]", "backtick_regex", "Bar"),

        // Additional test cases
        ("This struct is also not [bar](`Bar`)", "link_backticks_regex", "bar -> Bar"),
        ("See [`Option`] for details", "backtick_regex", "Option"),
        ("Check [custom text](SomeType) here", "link_no_backticks_regex", "custom text -> SomeType"),
        ("Reference [text][Reference] style", "reference_style_regex", "text -> Reference"),

        // Reference definition (should match)
        ("[b]: Bar", "reference_def_regex", "b -> Bar"),
        ("  [reference]: Target", "reference_def_regex", "reference -> Target"),

        // Edge cases that should NOT match certain patterns
        ("This [has] [nested] brackets", "simple_regex", "has (should match)"),
        ("This [text](link) and [more](links)", "link_no_backticks_regex", "multiple matches"),
    ];

    for (input, expected_pattern, description) in test_cases {
        println!("Testing: {}", input);
        println!("Expected: {} - {}", expected_pattern, description);

        // Test each regex
        test_regex(&backtick_regex, "backtick_regex", input);
        test_regex(&simple_regex, "simple_regex", input);
        test_regex(&link_backticks_regex, "link_backticks_regex", input);
        test_regex(&link_no_backticks_regex, "link_no_backticks_regex", input);
        test_regex(&reference_style_regex, "reference_style_regex", input);
        test_regex(&reference_def_regex, "reference_def_regex", input);

        println!();
    }

    // Test the complete processing pipeline
    println!("=== Testing complete pipeline ===");
    test_complete_pipeline();
}

fn test_regex(regex: &Regex, name: &str, input: &str) {
    if let Some(caps) = regex.captures(input) {
        print!("  {} MATCH: ", name);
        if let Some(text) = caps.name("text") {
            print!("text='{}' ", text.as_str());
        }
        if let Some(reference) = caps.name("ref") {
            print!("ref='{}'", reference.as_str());
        }
        if let Some(target) = caps.name("target") {
            print!("target='{}'", target.as_str());
        }
        println!();
    }
}

fn test_complete_pipeline() {
    let test_docs = vec![
        "This struct is not [Bar]",
        "This struct is also not [bar](Bar)",
        "This struct is also not [bar][b]\n\n[b]: Bar",
        "This struct is also not [`Bar`]",
        "This struct *is* [`Bar`]!",
        "See [Bar], [`Option`], [custom text](Bar), and [other][Option] for details.",
        "Unlike normal Markdown, [bar][Bar] syntax is also supported.",
        "Backticks around the link will be stripped, so [`Option`] will correctly link to Option.",
    ];

    for doc in test_docs {
        println!("Input:  {}", doc.replace('\n', "\\n"));
        let processed = process_doc_references(doc);
        println!("Output: {}", processed.replace('\n', "\\n"));
        println!();
    }
}

fn process_doc_references(text: &str) -> String {
    let backtick_regex = Regex::new(r"\[`(?P<ref>[^`\]]+)`\]").unwrap();
    let simple_regex = Regex::new(r"\[(?P<ref>[^\]`\(\)]+)\](?!\(|\[)").unwrap();
    let link_backticks_regex = Regex::new(r"\[(?P<text>[^\]]+)\]\(`(?P<ref>[^`\)]+)`\)").unwrap();
    let link_no_backticks_regex = Regex::new(r"\[(?P<text>[^\]]+)\]\((?P<ref>[^\)`]+)\)").unwrap();
    let reference_style_regex = Regex::new(r"\[(?P<text>[^\]]+)\]\[(?P<ref>[^\]]+)\]").unwrap();
    let reference_def_regex = Regex::new(r"^\s*\[(?P<ref>[^\]]+)\]:\s*(?P<target>.+)$").unwrap();

    let mut result = text.to_string();

    // Process in order of specificity (most specific first)

    // Handle [`reference`] style - backticks around the link will be stripped
    result = backtick_regex.replace_all(&result, |caps: &regex::Captures| {
        let reference = &caps["ref"];
        format!("[{}]({})", reference, reference) // Simplified transformation
    }).to_string();

    // Handle [text](`reference`) style - link with backticks
    result = link_backticks_regex.replace_all(&result, |caps: &regex::Captures| {
        let text = &caps["text"];
        let reference = &caps["ref"];
        format!("[{}]({})", text, reference)
    }).to_string();

    // Handle [text](reference) style - link without backticks
    result = link_no_backticks_regex.replace_all(&result, |caps: &regex::Captures| {
        let text = &caps["text"];
        let reference = &caps["ref"];
        format!("[{}]({})", text, reference) // Keep as-is, already correct format
    }).to_string();

    // Handle [text][reference] style - reference-style link
    result = reference_style_regex.replace_all(&result, |caps: &regex::Captures| {
        let text = &caps["text"];
        let reference = &caps["ref"];
        format!("[{}]({})", text, reference)
    }).to_string();

    // Handle [reference] style - simple link
    result = simple_regex.replace_all(&result, |caps: &regex::Captures| {
        let reference = &caps["ref"];
        format!("[{}]({})", reference, reference)
    }).to_string();

    // Remove reference definitions
    result = reference_def_regex.replace_all(&result, "").to_string();

    result
}
