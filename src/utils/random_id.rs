use rand::Rng;

const WORDS: &str = "abcdefghijklmnopqrstuvwxyz";

pub fn random_id() -> String {
    let mut string = String::new();

    for _i in 0..24 {
        let random_num = rand::thread_rng().gen_range(0..2);

        if random_num == 0 {
            let rando = rand::thread_rng().gen_range(0..WORDS.len());
            let char = if let Some(chr) = WORDS.chars().nth(rando) {
                chr
            } else {
                'a'
            };

            string.push(char)
        } else if random_num == 1 {
            let rando = rand::thread_rng().gen_range(0..10);

            string.push_str(&rando.to_string())
        }
    }

    string
}
