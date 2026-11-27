/// Extrai o primeiro GUID (minúsculo) de saídas como
/// "Power Scheme GUID: 381b4222-f694-41f0-9685-ff5bb260df2e  (Balanced)".
/// O texto muda com o idioma do Windows; o GUID não.
pub fn parse_guid(output: &str) -> Option<String> {
    output
        .split(|c: char| !(c.is_ascii_hexdigit() || c == '-'))
        .find(|t| t.len() == 36 && t.matches('-').count() == 4)
        .map(|s| s.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_english_and_portuguese() {
        assert_eq!(
            parse_guid("Power Scheme GUID: 381b4222-f694-41f0-9685-ff5bb260df2e  (Balanced)")
                .as_deref(),
            Some("381b4222-f694-41f0-9685-ff5bb260df2e")
        );
        assert_eq!(
            parse_guid("GUID do Esquema de Energia: E9A42B02-D5DF-448D-AA00-03F14749EB61  (Desempenho Máximo)")
                .as_deref(),
            Some("e9a42b02-d5df-448d-aa00-03f14749eb61")
        );
        assert_eq!(parse_guid("nada aqui"), None);
    }
}
