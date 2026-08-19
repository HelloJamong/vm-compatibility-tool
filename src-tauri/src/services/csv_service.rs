/// CSV 필드를 RFC 4180 형태로 이스케이프하고 스프레드시트 수식 실행을 방지합니다.
pub fn escape_field(field: &str) -> String {
    let first_visible = field.trim_start().chars().next();
    let safe = if matches!(first_visible, Some('=' | '+' | '-' | '@')) {
        format!("'{field}")
    } else {
        field.to_string()
    };

    if safe.contains(',') || safe.contains('\n') || safe.contains('\r') || safe.contains('"') {
        format!("\"{}\"", safe.replace('"', "\"\""))
    } else {
        safe
    }
}

#[cfg(test)]
mod tests {
    use super::escape_field;

    #[test]
    fn spreadsheet_formula_prefixes_are_neutralized() {
        assert_eq!(escape_field("=1+1"), "'=1+1");
        assert_eq!(escape_field("  @SUM(A1:A2)"), "'  @SUM(A1:A2)");
    }

    #[test]
    fn csv_delimiters_and_quotes_are_escaped() {
        assert_eq!(escape_field("a,b"), "\"a,b\"");
        assert_eq!(escape_field("a\"b"), "\"a\"\"b\"");
    }

    #[test]
    fn ordinary_text_is_unchanged() {
        assert_eq!(escape_field("Windows 11"), "Windows 11");
    }
}
