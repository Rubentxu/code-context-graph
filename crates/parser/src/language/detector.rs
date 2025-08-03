use code_context_graph_core::{Language, Result, CodeGraphError};
use std::path::Path;
use std::fs;

pub struct LanguageDetector;

impl LanguageDetector {
    pub fn detect_from_path(path: &Path) -> Language {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            match extension.to_lowercase().as_str() {
                "py" => Language::Python,
                "java" => Language::Java,
                "kt" | "kts" => Language::Kotlin,
                "js" | "mjs" => Language::JavaScript,
                "ts" => Language::TypeScript,
                _ => Language::Unknown,
            }
        } else {
            Language::Unknown
        }
    }

    pub fn detect_from_content(content: &str) -> Language {
        // Simple heuristics for content-based detection
        let content_lower = content.to_lowercase();
        
        // Python indicators
        if content_lower.contains("def ") || content_lower.contains("import ") 
            || content_lower.contains("from ") || content_lower.contains("class ") 
            || content_lower.contains("print(") || content_lower.contains("__init__") {
            // Additional Python-specific checks to avoid false positives
            if !content_lower.contains("{") && !content_lower.contains("public ") {
                return Language::Python;
            }
        }
        
        // Java indicators
        if content_lower.contains("public class") || content_lower.contains("package ")
            || (content_lower.contains("import ") && content_lower.contains("{")) {
            return Language::Java;
        }
        
        // Kotlin indicators
        if content_lower.contains("fun ") || content_lower.contains("val ") 
            || content_lower.contains("var ") {
            return Language::Kotlin;
        }
        
        // JavaScript/TypeScript indicators
        if content_lower.contains("function ") || content_lower.contains("const ")
            || content_lower.contains("let ") || content_lower.contains("var ") {
            if content_lower.contains("interface ") || content_lower.contains(": ") {
                return Language::TypeScript;
            }
            return Language::JavaScript;
        }
        
        Language::Unknown
    }

    pub fn detect_from_file(path: &Path) -> Result<Language> {
        // First try by extension
        let lang_by_ext = Self::detect_from_path(path);
        if lang_by_ext != Language::Unknown {
            return Ok(lang_by_ext);
        }

        // Fallback to content analysis for ambiguous cases
        let content = fs::read_to_string(path)?;
        let lang_by_content = Self::detect_from_content(&content);
        
        Ok(lang_by_content)
    }

    pub fn is_supported(language: &Language) -> bool {
        matches!(language, 
            Language::Python | 
            Language::Java | 
            Language::Kotlin | 
            Language::JavaScript |
            Language::TypeScript
        )
    }

    pub fn get_file_patterns(language: &Language) -> Vec<&'static str> {
        match language {
            Language::Python => vec!["*.py", "*.pyw"],
            Language::Java => vec!["*.java"],
            Language::Kotlin => vec!["*.kt", "*.kts"],
            Language::JavaScript => vec!["*.js", "*.mjs"],
            Language::TypeScript => vec!["*.ts", "*.tsx"],
            Language::Unknown => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_from_extension() {
        assert_eq!(LanguageDetector::detect_from_path(&PathBuf::from("test.py")), Language::Python);
        assert_eq!(LanguageDetector::detect_from_path(&PathBuf::from("test.java")), Language::Java);
        assert_eq!(LanguageDetector::detect_from_path(&PathBuf::from("test.kt")), Language::Kotlin);
        assert_eq!(LanguageDetector::detect_from_path(&PathBuf::from("test.js")), Language::JavaScript);
        assert_eq!(LanguageDetector::detect_from_path(&PathBuf::from("test.ts")), Language::TypeScript);
        assert_eq!(LanguageDetector::detect_from_path(&PathBuf::from("README")), Language::Unknown);
    }

    #[test]
    fn test_detect_from_content() {
        let python_code = "def hello():\n    print('Hello world')";
        assert_eq!(LanguageDetector::detect_from_content(python_code), Language::Python);

        let java_code = "public class Hello {\n    public static void main(String[] args) {}\n}";
        assert_eq!(LanguageDetector::detect_from_content(java_code), Language::Java);

        let kotlin_code = "fun main() {\n    val message = \"Hello\"\n}";
        assert_eq!(LanguageDetector::detect_from_content(kotlin_code), Language::Kotlin);

        let js_code = "function hello() {\n    console.log('Hello');\n}";
        assert_eq!(LanguageDetector::detect_from_content(js_code), Language::JavaScript);
    }

    #[test]
    fn test_is_supported() {
        assert!(LanguageDetector::is_supported(&Language::Python));
        assert!(LanguageDetector::is_supported(&Language::Java));
        assert!(LanguageDetector::is_supported(&Language::Kotlin));
        assert!(LanguageDetector::is_supported(&Language::JavaScript));
        assert!(!LanguageDetector::is_supported(&Language::Unknown));
    }
}