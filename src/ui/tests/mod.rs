#[cfg(test)]
mod tests {
    use crate::transformers::TransformType;
    use crate::ui::terminal::AppMode;
    use crate::ui::theme::Theme;

    #[test]
    fn test_theme_default() {
        let theme = Theme::default();
        // Verify default theme has expected colors
        assert!(
            theme.explorer_directory != theme.explorer_file,
            "Directory and file styles should be different"
        );
    }

    #[test]
    fn test_app_mode_default() {
        let mode = AppMode::default();
        assert_eq!(mode, AppMode::Normal, "Default app mode should be Normal");
    }

    #[test]
    fn test_transform_type() {
        use crate::transformers::transform;

        let test_filename = "test-file_example.txt";
        let result = transform(test_filename, &TransformType::Snake);

        assert!(result.contains('_'));
        assert!(!result.contains('-'));
    }
}
