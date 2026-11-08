pub fn normalize(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut normalized = String::with_capacity(text.len());
    let mut separated = false;

    for (index, char) in text.char_indices() {
        let is_decimal_point = char == '.'
            && index > 0
            && bytes[index - 1].is_ascii_digit()
            && bytes
                .get(index + 1)
                .is_some_and(|byte| byte.is_ascii_digit());
        if char.is_ascii_alphanumeric()
            || matches!(char, '<' | '>' | '=' | '+' | '%')
            || is_decimal_point
        {
            if separated && !normalized.is_empty() {
                normalized.push(' ');
            }
            normalized.push(char.to_ascii_lowercase());
            separated = false;
        } else {
            separated = true;
        }
    }

    normalized
}
