#[cfg(test)]
mod tests {
    use crate::ui::terminal::app::{FileOperation, TransformType};
    use crate::ui::terminal::AppMode;
    use crate::ui::theme::Theme;
    use std::path::PathBuf;

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
        let test_filename = "test-file_example.txt";

        // Test snake case transform
        let operation = FileOperation {
            source: PathBuf::from("/tmp/test-file_example.txt"),
            destination: PathBuf::from("/tmp/test_file_example.txt"),
            operation_type: crate::ui::terminal::app::OperationType::Transform(
                TransformType::Snake,
            ),
        };

        assert_eq!(
            operation.source.file_name().unwrap().to_str().unwrap(),
            test_filename
        );
        assert!(operation
            .destination
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .contains('_'));
        assert!(!operation
            .destination
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .contains('-'));
    }
}
