mod tests {
    #[test]
    fn test_api_version() {
        let vsapi = crate::vsscript::ScriptAPI::get().unwrap();
        let version = vsapi.get_api_version();
        assert!(version >= crate::ffi::VSSCRIPT_API_VERSION)
    }
}