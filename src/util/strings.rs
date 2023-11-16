pub fn to_upper_camel_case(input: &str) -> String {
    let words: Vec<&str> = input.split([' ', '_', '-'].as_ref()).collect();
    let upper_camel_case: String = words
        .into_iter()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first_char) => first_char.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect();
    upper_camel_case
}
