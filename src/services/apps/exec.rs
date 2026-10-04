/*
 * removes the field codes like %f and %U, which a launcher
 * fills with files to open, and a menu has none of
 */
pub fn strip_field_codes(exec: &str) -> String {
    let mut command = String::new();

    let mut characters = exec.chars();

    while let Some(character) = characters.next() {
        if character != '%' {
            command.push(character);

            continue;
        }

        // %% is a plain percent sign, anything else after % is dropped with it
        if let Some('%') = characters.next() {
            command.push('%');
        }
    }

    command.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_field_codes_but_keeps_double_percent() {
        assert_eq!(strip_field_codes("firefox %u"), "firefox");
        assert_eq!(strip_field_codes("code --new-window %F"), "code --new-window");
        assert_eq!(strip_field_codes("printf 100%%"), "printf 100%");
    }
}
