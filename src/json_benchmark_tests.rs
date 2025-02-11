#[cfg(test)]
mod tests {

    use crate::json_benchmark::request::replace_special_strings;
    use crate::json_benchmark::utils::{
        generate_random_number_string, generate_random_string, replace_random_strings,
    };
    use rand::rng;

    #[test]
    fn test_replace_special_strings() {
        let url = "http://example.com/$CNT";
        let result = replace_special_strings(url, 1);
        assert_eq!(result, "http://example.com/1");

        let url = "http://example.com/$RND(5)";
        let result = replace_special_strings(url, 1);
        assert_eq!(result.len(), 24); // 19 + 5 random characters
        assert!(result[19..].chars().all(|c| c.is_alphanumeric()));

        let url = "http://example.com/$NRND(3)";
        let result = replace_special_strings(url, 1);
        assert_eq!(result.len(), 22); // 19 + 3 random digits
        assert!(result[19..].chars().all(|c| c.is_digit(10)));

        let url = "http://example.com/$CNT/$RND(4)/$NRND(2)";
        let result = replace_special_strings(url, 1);
        println!("{}", &result);
        assert_eq!(result.len(), 28); // 19 + 1+1 + 4+1 + 2 random digits
    }

    #[test]
    fn test_generate_random_string() {
        let mut rng = rng();
        let random_string = generate_random_string(10, &mut rng);
        assert_eq!(random_string.len(), 10);
        assert!(random_string.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_generate_random_number_string() {
        let mut rng = rng();
        let random_number_string = generate_random_number_string(5, &mut rng);
        assert_eq!(random_number_string.len(), 5);
        assert!(random_number_string.chars().all(|c| c.is_digit(10)));
    }

    #[test]
    fn test_replace_random_strings() {
        let mut rng = rng();
        let url = "http://example.com/$RND(4)";
        let result = replace_random_strings(url, &mut rng, "$RND(", &generate_random_string);
        println!("{}", &result);

        assert_eq!(result.len(), 23); // 19 + 4 random characters
        assert!(result[19..].chars().all(|c| c.is_alphanumeric()));

        let url = "http://example.com/$NRND(3)";
        let result =
            replace_random_strings(url, &mut rng, "$NRND(", &generate_random_number_string);
        println!("{}", &result);

        assert_eq!(result.len(), 22); // 19 + 3 random digits
        assert!(result[19..].chars().all(|c| c.is_digit(10)));
    }
}
