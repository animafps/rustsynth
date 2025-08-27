#[cfg(test)]
mod tests {
    use crate::format::{ColorFamily, PresetFormat, SampleType};

    #[test]
    fn test_preset_format_values() {
        // Test some key preset format values
        assert_eq!(PresetFormat::None as i32, 0);
        assert_ne!(PresetFormat::Gray8 as i32, 0);
        assert_ne!(PresetFormat::YUV420P8 as i32, 0);
        assert_ne!(PresetFormat::RGB24 as i32, 0);
    }

    #[test]
    fn test_color_family_enum() {
        // Test ColorFamily enum values
        assert_eq!(ColorFamily::Undefined as i32, 0);
        assert_eq!(ColorFamily::Gray as i32, 1);
        assert_eq!(ColorFamily::RGB as i32, 2);
        assert_eq!(ColorFamily::YUV as i32, 3);
    }

    #[test]
    fn test_sample_type_enum() {
        // Test SampleType enum values
        assert_eq!(SampleType::Integer as i32, 0);
        assert_eq!(SampleType::Float as i32, 1);
    }

    #[test]
    fn test_format_id_uniqueness() {
        // Different formats should have different IDs
        assert_ne!(PresetFormat::Gray8 as i32, PresetFormat::Gray16 as i32);
        assert_ne!(PresetFormat::Gray8 as i32, PresetFormat::YUV420P8 as i32);
        assert_ne!(PresetFormat::RGB24 as i32, PresetFormat::YUV420P8 as i32);

        // Float vs Integer formats should be different
        assert_ne!(PresetFormat::GrayS as i32, PresetFormat::Gray32 as i32);
    }

    #[test]
    fn test_subsampling_differences() {
        // Different YUV subsampling should give different IDs
        assert_ne!(PresetFormat::YUV420P8 as i32, PresetFormat::YUV422P8 as i32);
        assert_ne!(PresetFormat::YUV422P8 as i32, PresetFormat::YUV444P8 as i32);
        assert_ne!(PresetFormat::YUV420P8 as i32, PresetFormat::YUV444P8 as i32);
    }
}
