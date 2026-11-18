/// Original, everyday prompts that can be represented without writing letters.
pub fn word_bank() -> Vec<String> {
    include_str!("words.txt")
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}
