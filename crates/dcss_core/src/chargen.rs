//! Character creation data: species and job definitions loaded from YAML.

use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct SpeciesDef {
    pub name: String,
    #[serde(default)]
    pub difficulty: String,
    #[serde(default)]
    pub str_field: Option<i32>,
    #[serde(default)]
    pub int_field: Option<i32>,
    #[serde(default)]
    pub dex_field: Option<i32>,
    #[serde(default)]
    pub recommended_jobs: Vec<String>,
    #[serde(default)]
    pub aptitudes: HashMap<String, i32>,
}

// Custom deserialize to handle "str"/"int"/"dex" field names (reserved in some contexts)
impl SpeciesDef {
    pub fn from_yaml(content: &str) -> Result<Self, serde_yaml::Error> {
        #[derive(Deserialize)]
        struct Raw {
            name: String,
            #[serde(default)]
            difficulty: String,
            #[serde(default, rename = "str")]
            str_val: Option<i32>,
            #[serde(default, rename = "int")]
            int_val: Option<i32>,
            #[serde(default, rename = "dex")]
            dex_val: Option<i32>,
            #[serde(default)]
            recommended_jobs: Vec<String>,
            #[serde(default)]
            aptitudes: HashMap<String, i32>,
        }
        let raw: Raw = serde_yaml::from_str(content)?;
        Ok(SpeciesDef {
            name: raw.name,
            difficulty: raw.difficulty,
            str_field: raw.str_val,
            int_field: raw.int_val,
            dex_field: raw.dex_val,
            recommended_jobs: raw.recommended_jobs,
            aptitudes: raw.aptitudes,
        })
    }

    pub fn str_stat(&self) -> i32 { self.str_field.unwrap_or(8) }
    pub fn int_stat(&self) -> i32 { self.int_field.unwrap_or(8) }
    pub fn dex_stat(&self) -> i32 { self.dex_field.unwrap_or(8) }
}

#[derive(Debug, Clone, Deserialize)]
pub struct JobDef {
    pub name: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub recommended_species: Vec<String>,
    #[serde(default)]
    pub skills: HashMap<String, i32>,
    #[serde(default)]
    pub equipment: Vec<String>,
}

impl JobDef {
    pub fn from_yaml(content: &str) -> Result<Self, serde_yaml::Error> {
        #[derive(Deserialize)]
        struct Raw {
            name: String,
            #[serde(default)]
            category: String,
            #[serde(default, rename = "str")]
            _str_val: Option<i32>,
            #[serde(default, rename = "int")]
            _int_val: Option<i32>,
            #[serde(default, rename = "dex")]
            _dex_val: Option<i32>,
            #[serde(default)]
            recommended_species: Vec<String>,
            #[serde(default)]
            skills: HashMap<String, i32>,
            #[serde(default)]
            equipment: Vec<String>,
        }
        let raw: Raw = serde_yaml::from_str(content)?;
        Ok(JobDef {
            name: raw.name,
            category: raw.category,
            recommended_species: raw.recommended_species,
            skills: raw.skills,
            equipment: raw.equipment,
        })
    }
}

#[derive(Resource, Default)]
pub struct SpeciesDefs(pub Vec<SpeciesDef>);

#[derive(Resource, Default)]
pub struct JobDefs(pub Vec<JobDef>);

/// The player's character creation choices.
#[derive(Resource, Default)]
pub struct ChargenState {
    pub species_index: usize,
    pub job_index: usize,
    pub confirmed: bool,
}

/// Load species and job definitions from YAML files.
pub fn load_chargen_data(species: &mut SpeciesDefs, jobs: &mut JobDefs) {
    let species_dir = "crawl-ref/source/dat/species";
    let job_dir = "crawl-ref/source/dat/jobs";

    if let Ok(entries) = std::fs::read_dir(species_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".yaml") || name.starts_with("deprecated") { continue }
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if let Ok(def) = SpeciesDef::from_yaml(&content) {
                    species.0.push(def);
                }
            }
        }
    }
    species.0.sort_by(|a, b| a.name.cmp(&b.name));

    if let Ok(entries) = std::fs::read_dir(job_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".yaml") || name.starts_with("deprecated") { continue }
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if let Ok(def) = JobDef::from_yaml(&content) {
                    jobs.0.push(def);
                }
            }
        }
    }
    jobs.0.sort_by(|a, b| a.name.cmp(&b.name));
}
