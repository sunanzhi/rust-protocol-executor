pub fn add_two(a: i32) -> i32 {
    internal_adder(a, 2)
}

fn internal_adder(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal() {
        assert_eq!(4, internal_adder(2, 2));
    }

    // 快速测试模板
    #[cfg(test)]
    mod yaml_to_obj {
        use serde_yaml;

        #[test]
        fn test_simple_case() {
            #[derive(Debug, serde::Deserialize)]
            struct Simple {
                value: String,
            }

            let yaml = "value: hello";
            let obj: Simple = serde_yaml::from_str(yaml).unwrap();
            tracing::info!("simple info: {:?}", obj);
            assert_eq!(obj.value, "hello");
        }
    }
}