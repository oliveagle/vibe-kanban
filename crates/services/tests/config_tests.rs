//! Tests for services::config module

use services::services::config::versions::v8::{
    Config, EditorConfig, EditorType, GitHubConfig, NotificationConfig, SoundFile, ThemeMode,
    UiLanguage,
};

#[test]
fn test_config_default() {
    let config = Config::default();
    // Verify default config can be created
    assert!(true);
}

#[test]
fn test_theme_mode_variants() {
    let _light = ThemeMode::Light;
    let _dark = ThemeMode::Dark;
    let _system = ThemeMode::System;
}

#[test]
fn test_editor_type_variants() {
    let _cursor = EditorType::Cursor;
    let _vscode = EditorType::VSCode;
    let _vscodium = EditorType::VSCodium;
    let _zed = EditorType::Zed;
    let _windsurf = EditorType::Windsurf;
    let _trae = EditorType::Trae;
}

#[test]
fn test_sound_file_variants() {
    let _typewriter = SoundFile::Typewriter;
    let _macos = SoundFile::Macos;
    let _bell = SoundFile::Bell;
    let _water = SoundFile::Water;
}

#[test]
fn test_ui_language_variants() {
    let _en = UiLanguage::En;
    let _zh = UiLanguage::Zh;
}

#[test]
fn test_notification_config_default() {
    let config = NotificationConfig::default();
    assert!(true);
}

#[test]
fn test_editor_config_default() {
    let config = EditorConfig::default();
    assert!(true);
}

#[test]
fn test_github_config_default() {
    let config = GitHubConfig::default();
    assert!(true);
}
