#[cfg(test)]
mod tests {
    use crate::api::API;
    use crate::map::OwnedMap;

    fn setup_api() -> API {
        API::get().expect("Failed to get VapourSynth API")
    }

    #[test]
    fn test_owned_map_creation() {
        let _api = setup_api();
        let map = OwnedMap::new();

        // Map should be empty initially
        assert_eq!(map.key_count(), 0);
    }

    #[test]
    fn test_map_set_and_get_int() {
        let _api = setup_api();
        let mut map = OwnedMap::new();

        // Set an integer value
        map.set("test_key", &42i64)
            .expect("Failed to set int value");

        // Verify the key exists and count is correct
        assert_eq!(map.key_count(), 1);

        // Get the value back
        let value: i64 = map.get("test_key").expect("Failed to get int value");
        assert_eq!(value, 42);

        // Test value count for this key
        assert_eq!(map.value_count("test_key").unwrap(), 1);
    }

    #[test]
    fn test_map_set_and_get_float() {
        let _api = setup_api();
        let mut map = OwnedMap::new();

        // Set a float value
        map.set("pi", &std::f64::consts::PI)
            .expect("Failed to set float value");

        // Get the value back
        let value: f64 = map.get("pi").expect("Failed to get float value");
        assert!((value - std::f64::consts::PI).abs() < f64::EPSILON);
    }

    #[test]
    fn test_map_set_and_get_string() {
        let _api = setup_api();
        let mut map = OwnedMap::new();

        // Set a string value
        let test_string = "Hello, VapourSynth!".to_string();
        map.set("message", &test_string)
            .expect("Failed to set string value");

        // Get the value back
        let value: String = map.get("message").expect("Failed to get string value");
        assert_eq!(value, test_string);
    }

    #[test]
    fn test_map_keys_iterator() {
        let _api = setup_api();
        let mut map = OwnedMap::new();

        // Add several key-value pairs
        map.set("key1", &1i64).unwrap();
        map.set("key2", &2i64).unwrap();
        map.set("key3", &3i64).unwrap();

        // Collect keys
        let keys: Vec<&str> = map.keys().collect();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1"));
        assert!(keys.contains(&"key2"));
        assert!(keys.contains(&"key3"));
    }

    #[test]
    fn test_map_key_existence() {
        let _api = setup_api();
        let mut map = OwnedMap::new();

        // Initially empty
        assert_eq!(map.key_count(), 0);

        // Add a key
        map.set("test", &1i64).unwrap();
        assert_eq!(map.key_count(), 1);

        // Check if key exists by trying to get it
        assert!(map.get::<i64>("test").is_ok());
        assert!(map.get::<i64>("nonexistent").is_err());
    }

    #[test]
    fn test_map_clear() {
        let _api = setup_api();
        let mut map = OwnedMap::new();

        // Add some data
        map.set("key1", &1i64).unwrap();
        map.set("key2", &2i64).unwrap();
        assert_eq!(map.key_count(), 2);

        // Clear the map
        map.clear();
        assert_eq!(map.key_count(), 0);
    }

    #[test]
    fn test_owned_map_macro() {
        let _api = setup_api();

        // Test the owned_map! macro
        let map = crate::owned_map! {
            {"int": &42i64},
            {"float": &std::f64::consts::PI},
            {"string": &"test".to_string()}
        };

        assert_eq!(map.key_count(), 3);
        assert_eq!(map.get::<i64>("int").unwrap(), 42);
        assert_eq!(map.get::<f64>("float").unwrap(), std::f64::consts::PI);
        assert_eq!(map.get::<String>("string").unwrap(), "test");
    }
}
