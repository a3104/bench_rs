use rand::{distr::Alphanumeric, Rng};
use uuid::Uuid;

pub fn generate_luid() -> String {
    Uuid::new_v4().to_string()
}

pub fn generate_random_hex_string(len: usize, rng: &mut impl Rng) -> String {
    (0..len).map(|_| format!("{:x}", rng.random_range(0..16))).collect::<String>()
}

pub fn generate_random_number_string(len: usize, rng: &mut impl Rng) -> String {
    (0..len).map(|_| rng.random_range(0..10).to_string()).collect::<String>()
}

pub fn generate_random_string(len: usize, rng: &mut impl Rng) -> String {
    (0..len).map(|_| rng.sample(Alphanumeric) as char).collect::<String>()
}

pub fn replace_random_strings<F, R>(url: &str, rng: &mut R, pattern: &str, generator: &F) -> String
where
    F: Fn(usize, &mut R) -> String,
    R: Rng,
{
    let mut replaced_url = url.to_string();
    while let Some(start_index) = replaced_url.find(pattern) {
        if let Some(end_index) = replaced_url[start_index..].find(")") {
            let length_str = &replaced_url[start_index + pattern.len()..start_index + end_index];
            match length_str.parse::<usize>() {
                Ok(length) => {
                    let generated_string = generator(length, rng);
                    replaced_url
                        .replace_range(start_index..start_index + end_index + 1, &generated_string);
                }
                Err(_) => break, // エラー処理
            }
        } else {
            break; // エラー処理
        }
    }
    replaced_url
}
