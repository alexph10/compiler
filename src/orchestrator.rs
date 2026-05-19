use crate::types::*;
use colored::Colorize;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

pub type BuildEntry = (DetectedProject, anyhow::Result<BuildResult>, bool, Duration);

pub fn resolve_build_order(projects: &[DetectedProject]) -> Vec<DetectedProject> {
    let mut graph: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut in_degree: HashMap<usize, usize> = HashMap::new();

    for i in 0..projects.len() {
        graph.entry(i).or_default();
        in_degree.entry(i).or_insert(0);
    }

    for i in 0..projects.len() {
        for j in 0..projects.len() {
            if i == j {
                continue;
            }
            if project_depends_on(&projects[i], &projects[j]) {
                graph.entry(j).or_default().push(i);
                *in_degree.entry(i).or_insert(0) += 1;
            }
        }
    }
    let mut queue: Vec<usize> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&idx, _)| idx)
        .collect();
    queue.sort();

    let mut ordered = Vec::new();
    while let Some(idx) = queue.pop() {
        ordered.push(idx);
        if let Some(dependents) = graph.get(&idx) {
            for &dep in dependents {
                if let Some(deg) = in_degree.get_mut(&dep) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push(dep);
                    }
                }
            }
        }
    }
    if ordered.len() != projects.len() {
        eprintln!(
            "{} circular dependency deteced, failing back to discovery order",
            "warning".yellow()
        );
        return projects.to_vec();
    }
    ordered.iter().map(|&i| projects[i].clone()).collect()
}

fn project_depends_on(a: &DetectedProject, b: &DetectedProject) -> bool {
    let a_path = &a.path;
    let b_path = &b.path;

    match a.language {
        Language::Rust => {
            let cargo_toml = a_path.join("Cargo.toml");
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                if let Some(b_name) = b_path.file_name().and_then(|n| n.to_str()) {
                    if content.contains("path = ")
                        && content.contains(
                            &b_path
                                .strip_prefix(a_path.parent().unwrap_or(a_path))
                                .unwrap_or(b_path)
                                .display()
                                .to_string(),
                        )
                    {
                        return true;
                    }
                    if content.contains(b_name) {
                        let dep_sections = [
                            "[dependencies]",
                            "[dev-dependencies]",
                            "[build-dependencies]",
                        ];
                        for section in &dep_sections {
                            if let Some(idx) = content.find(section) {
                                let section_content = &content[idx..];
                                if let Some(end) = section_content[1..].find("[") {
                                    let slice = &section_content[..end + 1];
                                    if slice.contains(b_name) && slice.contains("path") {
                                        return true;
                                    }
                                } else if section_content.contains(b_name)
                                    && section_content.contains("path")
                                {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        Language::Go => {
            let go_mod = a_path.join("go.mod");
            if let Ok(content) = std::fs::read_to_string(&go_mod) {
                if content.contains("replace") {
                    if let Some(b_name) = b_path.file_name().and_then(|n| n.to_str()) {
                        if content.contains(b_name) {
                            return true;
                        }
                    }
                }
            }
        }
        Language::TypeScript => {
            let pkg_json = a_path.join("package.json");
            if let Ok(content) = std::fs::read_to_string(&pkg_json) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    for section in &["dependencies", "devDependencies"] {
                        if let Some(deps) = val.get(section).and_then(|d| d.as_object()) {
                            for (_, v) in deps {
                                if let Some(s) = v.as_str() {
                                    if s.starts_with("file:")
                                        || s.starts_with("link:")
                                        || s.starts_with("workspace:")
                                    {
                                        let dep_path = s
                                            .trim_start_matches("file:")
                                            .trim_start_matches("link:")
                                            .trim_start_matches("workspace:");
                                        let resolved = a_path.join(dep_path);
                                        if resolved == *b_path
                                            || resolved.canonicalize().ok()
                                                == b.path.canonicalize().ok()
                                        {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Language::Python => {
            let pyproject = a_path.join("pyproject.toml");
            if let Ok(content) = std::fs::read_to_string(&pyproject) {
                if let Some(b_name) = b_path.file_name().and_then(|n| n.to_str()) {
                    if content.contains(b_name) && content.contains("path") {
                        return true;
                    }
                }
            }
        }
        Language::C => {
            let cmake = a_path.join("CMakeLists.txt");
            if let Ok(content) = std::fs::read_to_string(&cmake) {
                if let Some(b_name) = b_path.file_name().and_then(|n| n.to_str()) {
                    if content.contains("add_subdirectory") && content.contains(b_name) {
                        return true;
                    }
                }
            }
        }
        Language::Zig => {
            let build_zig = a_path.join("build.zig");
            if let Ok(content) = std::fs::read_to_string(&build_zig) {
                if let Some(b_name) = b_path.file_name().and_then(|n| n.to_str()) {
                    if content.contains("dependency") && content.contains(b_name) {
                        return true;
                    }
                }
            }
        }
    }
    false
}
