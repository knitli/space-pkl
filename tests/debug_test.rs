fn type_name_to_pkl(name: &str) -> String {
    // Remove common Rust type prefixes/suffixes
    println!("Input: '{}'", name);
    let step1 = name.trim_end_matches("Config");
    println!("After removing Config: '{}'", step1);
    let step2 = step1.trim_end_matches("Type");
    println!("After removing Type: '{}'", step2);
    let cleaned = step2.trim_start_matches("Partial");
    println!("After removing Partial: '{}'", cleaned);

    capitalize_first_letter(cleaned)
}

fn capitalize_first_letter(s: &str) -> String {
    s.chars()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.to_uppercase().collect() } else { c.to_string() })
        .collect()
}

fn main() {
    println!("Testing PartialPartialConfig:");
    let result = type_name_to_pkl("PartialPartialConfig");
    println!("Result: '{}'", result);
}
