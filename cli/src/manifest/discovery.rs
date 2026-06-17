use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ManifestKind {
    CloudApps,
    Auth,
    Iac,
    Unsupported,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ManifestSource {
    pub(crate) path: PathBuf,
    pub(crate) kind: ManifestKind,
    pub(crate) detail: String,
}

pub(crate) fn discover(explicit_file: Option<&Path>, cwd: &Path) -> Result<Vec<ManifestSource>> {
    if let Some(file) = explicit_file {
        let path = absolutize(cwd, file);
        return classify_file(&path);
    }

    let mut sources = Vec::new();
    let project_root = if let Some(path) = find_tachyon_yml(cwd) {
        sources.extend(classify_file(&path)?);
        path.parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| cwd.to_path_buf())
    } else {
        find_repo_root(cwd).unwrap_or_else(|| cwd.to_path_buf())
    };

    let manifests_dir = project_root.join(".tachyon").join("manifests");
    if manifests_dir.is_dir() {
        let mut paths = collect_yaml_files(&manifests_dir)?;
        paths.sort();
        for path in paths {
            sources.extend(classify_file(&path)?);
        }
    }

    sources.sort_by(|a, b| {
        source_order(a)
            .cmp(&source_order(b))
            .then(a.path.cmp(&b.path))
    });
    Ok(sources)
}

fn classify_file(path: &Path) -> Result<Vec<ManifestSource>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut sources = Vec::new();

    for doc in serde_yaml::Deserializer::from_str(&raw) {
        let value = Value::deserialize(doc).with_context(|| format!("parse {}", path.display()))?;
        if value.is_null() {
            continue;
        }
        classify_value(path, &value, &mut sources);
    }

    if sources.is_empty() {
        sources.push(ManifestSource {
            path: path.to_path_buf(),
            kind: ManifestKind::Unsupported,
            detail: "empty".to_string(),
        });
    }

    Ok(sources)
}

fn classify_value(path: &Path, value: &Value, sources: &mut Vec<ManifestSource>) {
    let is_cloud_apps = matches!(
        value.get("kind").and_then(Value::as_str),
        Some("CloudApps") | Some("CloudApp")
    );
    if is_cloud_apps {
        sources.push(ManifestSource {
            path: path.to_path_buf(),
            kind: ManifestKind::CloudApps,
            detail: "Cloud Apps".to_string(),
        });
        return;
    }

    if value
        .get("auth")
        .and_then(|auth| auth.get("manifest"))
        .is_some()
        || looks_like_auth(value)
    {
        sources.push(ManifestSource {
            path: path.to_path_buf(),
            kind: ManifestKind::Auth,
            detail: "auth manifest".to_string(),
        });
    } else if value.get("apiVersion").and_then(Value::as_str) == Some("apps.tachy.one/v1alpha") {
        sources.push(ManifestSource {
            path: path.to_path_buf(),
            kind: ManifestKind::Iac,
            detail: value
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
        });
    } else {
        sources.push(ManifestSource {
            path: path.to_path_buf(),
            kind: ManifestKind::Unsupported,
            detail: value
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
        });
    }
}

fn looks_like_auth(value: &Value) -> bool {
    value.get("actions").is_some()
        || value.get("policies").is_some()
        || value
            .get("apiVersion")
            .and_then(Value::as_str)
            .is_some_and(|version| version == "auth.tachyon.io/v1")
        || value
            .get("items")
            .and_then(Value::as_array)
            .is_some_and(|items| {
                items.iter().any(|item| {
                    item.get("apiVersion")
                        .and_then(Value::as_str)
                        .is_some_and(|version| version == "auth.tachyon.io/v1")
                })
            })
}

fn source_order(source: &ManifestSource) -> (u8, &Path) {
    let rank = match source.kind {
        ManifestKind::Iac => 0,
        ManifestKind::Auth => 1,
        ManifestKind::CloudApps => 2,
        ManifestKind::Unsupported => 3,
    };
    (rank, source.path.as_path())
}

fn find_tachyon_yml(cwd: &Path) -> Option<PathBuf> {
    let mut dir = cwd;
    loop {
        let candidate = dir.join("tachyon.yml");
        if candidate.is_file() {
            return Some(candidate);
        }
        if dir.join(".git").exists() {
            return None;
        }
        dir = dir.parent()?;
    }
}

fn find_repo_root(cwd: &Path) -> Option<PathBuf> {
    let mut dir = cwd;
    loop {
        if dir.join(".git").exists() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

fn collect_yaml_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("read dir {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            result.extend(collect_yaml_files(&path)?);
        } else if matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("yml" | "yaml")
        ) {
            result.push(path);
        }
    }
    Ok(result)
}

fn absolutize(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn discover_uses_single_file_when_explicit_file_is_set() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join(".git")).unwrap();
        write(
            &tmp.path().join("tachyon.yml"),
            "kind: CloudApps\nspec:\n  apps: []\n",
        );
        write(
            &tmp.path().join(".tachyon/manifests/auth.yml"),
            "actions:\n  - context: auth\n    name: Read\npolicies: []\n",
        );

        let sources = discover(Some(Path::new(".tachyon/manifests/auth.yml")), tmp.path()).unwrap();

        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].kind, ManifestKind::Auth);
        assert!(sources[0].path.ends_with(".tachyon/manifests/auth.yml"));
    }

    #[test]
    fn discover_orders_iac_auth_then_cloud_apps() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join(".git")).unwrap();
        write(
            &tmp.path().join("tachyon.yml"),
            "kind: CloudApps\nspec:\n  apps: []\n",
        );
        write(
            &tmp.path().join(".tachyon/manifests/z-auth.yml"),
            "actions:\n  - context: auth\n    name: Read\npolicies: []\n",
        );
        write(
            &tmp.path().join(".tachyon/manifests/a-iac.yml"),
            "apiVersion: apps.tachy.one/v1alpha\nkind: Operator\nmetadata:\n  name: op\n",
        );

        let sources = discover(None, tmp.path()).unwrap();
        let kinds = sources
            .iter()
            .map(|source| source.kind.clone())
            .collect::<Vec<_>>();

        assert_eq!(
            kinds,
            vec![
                ManifestKind::Iac,
                ManifestKind::Auth,
                ManifestKind::CloudApps
            ]
        );
    }

    #[test]
    fn discover_classifies_multi_document_tachyon_yml() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join(".git")).unwrap();
        write(
            &tmp.path().join("tachyon.yml"),
            r#"apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
spec:
  apps:
    - name: fieldadmin
---
apiVersion: apps.tachy.one/v1alpha
kind: OAuth2Client
metadata:
  name: fieldadmin-web
"#,
        );

        let sources = discover(None, tmp.path()).unwrap();
        let kinds = sources
            .iter()
            .map(|source| source.kind.clone())
            .collect::<Vec<_>>();

        assert_eq!(kinds, vec![ManifestKind::Iac, ManifestKind::CloudApps]);
    }

    #[test]
    fn discover_uses_repo_root_manifests_without_tachyon_yml() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join(".git")).unwrap();
        let nested = tmp.path().join("apps/api");
        fs::create_dir_all(&nested).unwrap();
        write(
            &tmp.path().join(".tachyon/manifests/auth.yml"),
            "actions:\n  - context: root\n    name: Read\npolicies: []\n",
        );

        let sources = discover(None, &nested).unwrap();

        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].kind, ManifestKind::Auth);
        assert!(sources[0].path.ends_with(".tachyon/manifests/auth.yml"));
    }
}
