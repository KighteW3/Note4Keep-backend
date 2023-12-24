use std::collections::HashMap;
use std::env;

const IMPORTANT_KEYS: [&str; 3] = ["SECRET", "PORT", "MONGODB_URI"];

pub fn check_integrity() {
    let all_vars = env::vars();
    let mut comprobe = HashMap::new();

    for key in IMPORTANT_KEYS {
        comprobe.insert(key, false);
    }

    for (key, _val) in all_vars {
        for ik in IMPORTANT_KEYS {
            if key == ik {
                comprobe.insert(ik, true);
            }
        }
    }

    for (key, val) in comprobe {
        if !val {
            panic!(
                "Error: the key {:?} not found in .env, please add the key with a valid value in a .env file at the root of the crate.",
                key
            );
        }
    }
}
