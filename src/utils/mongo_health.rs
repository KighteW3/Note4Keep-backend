const SPECIAL_CHARS: &str = "*/!-,`[";

pub fn mongo_query_error(input: String) -> bool {
    let mut to_return: bool = false;

    for char in SPECIAL_CHARS.chars() {
        match input.contains(char) {
            true => {
                to_return = true;
                break;
            }
            _ => {}
        };
    }

    to_return
}
