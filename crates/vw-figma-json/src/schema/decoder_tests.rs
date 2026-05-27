    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_kiwi_value_to_json_bool() {
        let value = Value::Bool(true);
        let json = kiwi_value_to_json(&value);
        assert_eq!(json, JsonValue::Bool(true));
    }

    #[test]
    fn test_kiwi_value_to_json_int() {
        let value = Value::Int(42);
        let json = kiwi_value_to_json(&value);
        assert_eq!(json, JsonValue::Number(42.into()));
    }

    #[test]
    fn test_kiwi_value_to_json_string() {
        let value = Value::String("hello".to_string());
        let json = kiwi_value_to_json(&value);
        assert_eq!(json, JsonValue::String("hello".to_string()));
    }

    #[test]
    fn test_kiwi_value_to_json_array() {
        let value = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let json = kiwi_value_to_json(&value);
        assert_eq!(
            json,
            JsonValue::Array(vec![
                JsonValue::Number(1.into()),
                JsonValue::Number(2.into()),
                JsonValue::Number(3.into()),
            ])
        );
    }

    #[test]
    fn test_kiwi_value_to_json_object() {
        let mut fields = HashMap::new();
        fields.insert("x", Value::Int(10));
        fields.insert("y", Value::Int(20));

        let value = Value::Object("Point", fields);
        let json = kiwi_value_to_json(&value);

        match json {
            JsonValue::Object(map) => {
                assert_eq!(map.get("x"), Some(&JsonValue::Number(10.into())));
                assert_eq!(map.get("y"), Some(&JsonValue::Number(20.into())));
            }
            _ => panic!("Expected JSON object"),
        }
    }
