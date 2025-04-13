use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use tauri::path::AppDataDirPathResolver;

// template categories
pub enum TemplateCategory {
    Daily,
    Weekly,
    Goals,
    Notes,
    Custom,
}

impl TemplateCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            TemplateCategory::Daily => "daily",
            TemplateCategory::Weekly => "weekly",
            TemplateCategory::Goals => "goals",
            TemplateCategory::Notes => "notes",
            TemplateCategory::Custom => "custom",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "daily" => Some(TemplateCategory::Daily),
            "weekly" => Some(TemplateCategory::Weekly),
            "goals" => Some(TemplateCategory::Goals),
            "notes" => Some(TemplateCategory::Notes),
            "custom" => Some(TemplateCategory::Custom),
            _ => None,
        }
    }

    pub fn all_categories() -> Vec<TemplateCategory> {
        vec![
            TemplateCategory::Daily,
            TemplateCategory::Weekly,
            TemplateCategory::Goals,
            TemplateCategory::Notes,
            TemplateCategory::Custom,
        ]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateInfo {
    pub name: String,
    pub path: String,
    pub category: String,
    pub created_at: u64,
    pub modified_at: u64,
}

// template manager for handling template file operations
pub struct TemplateManager {
    templates_dir: PathBuf,
}

impl TemplateManager {
    // new template manager with the app data directory
    pub fn new(app_data_dir: &Path) -> Self {
        let templates_dir = app_data_dir.join("templates");
        Self { templates_dir }
    }

    pub fn init_directories(&self) -> io::Result<()> {
        // Cceate the main templates directory if it doesn't exist
        if !self.templates_dir.exists() {
            fs::create_dir_all(&self.templates_dir)?;
        }

        // create subdirectories for each category
        for category in TemplateCategory::all_categories() {
            let category_dir = self.templates_dir.join(category.as_str());
            if !category_dir.exists() {
                fs::create_dir_all(&category_dir)?;
            }
        }

        Ok(())
    }

    // path to a template category directory
    pub fn get_category_path(&self, category: &TemplateCategory) -> PathBuf {
        self.templates_dir.join(category.as_str())
    }

    // save template to a file
    pub fn save_template(
        &self,
        category: &TemplateCategory,
        name: &str,
        content: &str,
    ) -> io::Result<PathBuf> {
        let category_dir = self.get_category_path(category);
        let file_path = category_dir.join(format!("{}.json", name));

        let mut file = fs::File::create(&file_path)?;
        file.write_all(content.as_bytes())?;

        Ok(file_path)
    }

    // read template from file
    pub fn read_template(&self, category: &TemplateCategory, name: &str) -> io::Result<String> {
        let category_dir = self.get_category_path(category);
        let file_path = category_dir.join(format!("{}.json", name));

        let mut file = fs::File::open(&file_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        Ok(content)
    }

    // all templates in a category
    pub fn list_templates(&self, category: &TemplateCategory) -> io::Result<Vec<TemplateInfo>> {
        let category_dir = self.get_category_path(category);
        let category_str = category.as_str();

        let mut templates = Vec::new();

        if !category_dir.exists() {
            return Ok(templates);
        }

        for entry in fs::read_dir(category_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                let metadata = fs::metadata(&path)?;
                let file_name = path.file_stem().unwrap().to_string_lossy().to_string();

                templates.push(TemplateInfo {
                    name: file_name,
                    path: path.to_string_lossy().to_string(),
                    category: category_str.to_string(),
                    created_at: metadata.created().map_or(0, |time| {
                        time.duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    }),
                    modified_at: metadata.modified().map_or(0, |time| {
                        time.duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    }),
                });
            }
        }

        Ok(templates)
    }

    // delete template
    pub fn delete_template(&self, category: &TemplateCategory, name: &str) -> io::Result<()> {
        let category_dir = self.get_category_path(category);
        let file_path = category_dir.join(format!("{}.json", name));

        if file_path.exists() {
            fs::remove_file(file_path)?;
        }

        Ok(())
    }
}

// tauri command wrappers
#[tauri::command]
pub fn init_template_directories(
    app_data_dir_resolver: tauri::State<'_, AppDataDirPathResolver>,
) -> Result<(), String> {
    let app_data_dir = app_data_dir_resolver
        .resolve()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;

    let template_manager = TemplateManager::new(&app_data_dir);
    template_manager
        .init_directories()
        .map_err(|e| format!("Failed to initialize template directories: {}", e))
}

#[tauri::command]
pub fn save_template_file(
    app_data_dir_resolver: tauri::State<'_, AppDataDirPathResolver>,
    category: String,
    name: String,
    content: String,
) -> Result<String, String> {
    let app_data_dir = app_data_dir_resolver
        .resolve()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;

    let template_category = TemplateCategory::from_str(&category)
        .ok_or_else(|| format!("Invalid template category: {}", category))?;

    let template_manager = TemplateManager::new(&app_data_dir);
    let path = template_manager
        .save_template(&template_category, &name, &content)
        .map_err(|e| format!("Failed to save template: {}", e))?;

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn read_template_file(
    app_data_dir_resolver: tauri::State<'_, AppDataDirPathResolver>,
    category: String,
    name: String,
) -> Result<String, String> {
    let app_data_dir = app_data_dir_resolver
        .resolve()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;

    let template_category = TemplateCategory::from_str(&category)
        .ok_or_else(|| format!("Invalid template category: {}", category))?;

    let template_manager = TemplateManager::new(&app_data_dir);
    template_manager
        .read_template(&template_category, &name)
        .map_err(|e| format!("Failed to read template: {}", e))
}

#[tauri::command]
pub fn list_template_files(
    app_data_dir_resolver: tauri::State<'_, AppDataDirPathResolver>,
    category: String,
) -> Result<Vec<TemplateInfo>, String> {
    let app_data_dir = app_data_dir_resolver
        .resolve()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;

    let template_category = TemplateCategory::from_str(&category)
        .ok_or_else(|| format!("Invalid template category: {}", category))?;

    let template_manager = TemplateManager::new(&app_data_dir);
    template_manager
        .list_templates(&template_category)
        .map_err(|e| format!("Failed to list templates: {}", e))
}

#[tauri::command]
pub fn delete_template_file(
    app_data_dir_resolver: tauri::State<'_, AppDataDirPathResolver>,
    category: String,
    name: String,
) -> Result<(), String> {
    let app_data_dir = app_data_dir_resolver
        .resolve()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;

    let template_category = TemplateCategory::from_str(&category)
        .ok_or_else(|| format!("Invalid template category: {}", category))?;

    let template_manager = TemplateManager::new(&app_data_dir);
    template_manager
        .delete_template(&template_category, &name)
        .map_err(|e| format!("Failed to delete template: {}", e))
}
