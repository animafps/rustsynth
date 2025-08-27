#[cfg(test)]
mod tests {
    use crate::ffi::VAPOURSYNTH_API_VERSION;

    use crate::api::API;
    use crate::core::{CoreCreationFlags, CoreRef};

    fn setup_api() -> API {
        API::get().expect("Failed to get VapourSynth API")
    }

    #[test]
    fn test_api_version() {
        let api = setup_api();
        let version = api.version();
        assert!(version >= VAPOURSYNTH_API_VERSION);
    }

    #[test]
    fn test_api_singleton() {
        // Test that API::get() returns the same instance
        let api1 = API::get().unwrap();
        let api2 = API::get().unwrap();

        // Both should return the same version
        assert_eq!(api1.version(), api2.version());
    }

    #[test]
    fn test_cached_api() {
        // Test that cached API works
        let api = API::get().unwrap();
        let cached_api = unsafe { API::get_cached() };

        assert_eq!(cached_api.version(), api.version());
    }

    #[test]
    fn test_create_core() {
        let _api = setup_api();

        // Test core creation with default flags
        let _core = CoreRef::new(CoreCreationFlags::NONE);
        // If we get here without panicking, core creation succeeded
    }

    #[test]
    fn test_core_creation_with_flags() {
        let _api = setup_api();

        // Test core creation with different flags
        let _core1 = CoreRef::new(CoreCreationFlags::ENABLE_GRAPH_INSPECTION);
        let _core2 = CoreRef::new(CoreCreationFlags::DISABLE_AUTO_LOADING);
        let _core3 = CoreRef::new(
            CoreCreationFlags::ENABLE_GRAPH_INSPECTION | CoreCreationFlags::DISABLE_AUTO_LOADING,
        );
        // If we get here without panicking, all core creations succeeded
    }
}
