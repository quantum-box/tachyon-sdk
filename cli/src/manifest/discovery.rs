use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
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
    pub(crate) id: String,
    pub(crate) depends_on: Vec<String>,
    #[serde(skip_serializing)]
    pub(crate) document: Value,
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

    sort_by_dependencies(sources)
}

fn classify_file(path: &Path) -> Result<Vec<ManifestSource>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut sources = Vec::new();

    for (document_index, doc) in serde_yaml::Deserializer::from_str(&raw).enumerate() {
        let value = Value::deserialize(doc).with_context(|| format!("parse {}", path.display()))?;
        if value.is_null() {
            continue;
        }
        classify_value(path, document_index, &value, &mut sources);
    }

    if sources.is_empty() {
        sources.push(ManifestSource {
            path: path.to_path_buf(),
            kind: ManifestKind::Unsupported,
            detail: "empty".to_string(),
            id: fallback_id(path, 0, "empty"),
            depends_on: Vec::new(),
            document: Value::Null,
        });
    }

    Ok(sources)
}

fn classify_value(
    path: &Path,
    document_index: usize,
    value: &Value,
    sources: &mut Vec<ManifestSource>,
) {
    let is_cloud_apps = matches!(
        value.get("kind").and_then(Value::as_str),
        Some("CloudApps") | Some("CloudApp")
    );
    if is_cloud_apps {
        let detail = "Cloud Apps".to_string();
        sources.push(ManifestSource {
            path: path.to_path_buf(),
            kind: ManifestKind::CloudApps,
            id: manifest_id(path, document_index, value, "CloudApps"),
            depends_on: manifest_dependencies(value),
            document: value.clone(),
            detail,
        });
        return;
    }

    if value
        .get("auth")
        .and_then(|auth| auth.get("manifest"))
        .is_some()
        || looks_like_auth(value)
    {
        let detail = "auth manifest".to_string();
        sources.push(ManifestSource {
            path: path.to_path_buf(),
            kind: ManifestKind::Auth,
            id: manifest_id(path, document_index, value, "AuthManifest"),
            depends_on: manifest_dependencies(value),
            document: value.clone(),
            detail,
        });
    } else if value.get("apiVersion").and_then(Value::as_str) == Some("apps.tachy.one/v1alpha") {
        let detail = value
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        sources.push(ManifestSource {
            path: path.to_path_buf(),
            kind: ManifestKind::Iac,
            id: manifest_id(path, document_index, value, &detail),
            depends_on: manifest_dependencies(value),
            document: value.clone(),
            detail,
        });
    } else {
        let detail = value
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        sources.push(ManifestSource {
            path: path.to_path_buf(),
            kind: ManifestKind::Unsupported,
            id: manifest_id(path, document_index, value, &detail),
            depends_on: manifest_dependencies(value),
            document: value.clone(),
            detail,
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

fn sort_by_dependencies(mut sources: Vec<ManifestSource>) -> Result<Vec<ManifestSource>> {
    sources.sort_by(|a, b| {
        source_order(a)
            .cmp(&source_order(b))
            .then(a.path.cmp(&b.path))
            .then(a.id.cmp(&b.id))
    });

    let mut id_to_index = HashMap::new();
    let mut name_to_indexes: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, source) in sources.iter().enumerate() {
        if id_to_index.insert(source.id.clone(), index).is_some() {
            return Err(anyhow!("duplicate manifest id '{}'", source.id));
        }
        if let Some(name) = source.id.split('/').nth(1) {
            name_to_indexes
                .entry(name.to_string())
                .or_default()
                .push(index);
        }
    }

    let mut dependents = vec![Vec::<usize>::new(); sources.len()];
    let mut indegree = vec![0usize; sources.len()];
    for (source_index, source) in sources.iter().enumerate() {
        for dependency in &source.depends_on {
            let dependency_index =
                resolve_dependency(dependency, &id_to_index, &name_to_indexes, source)?;
            dependents[dependency_index].push(source_index);
            indegree[source_index] += 1;
        }
    }

    let mut ready = indegree
        .iter()
        .enumerate()
        .filter_map(|(index, degree)| (*degree == 0).then_some(index))
        .collect::<VecDeque<_>>();
    let mut ordered_indexes = Vec::with_capacity(sources.len());
    while let Some(index) = ready.pop_front() {
        ordered_indexes.push(index);
        for dependent in &dependents[index] {
            indegree[*dependent] -= 1;
            if indegree[*dependent] == 0 {
                ready.push_back(*dependent);
            }
        }
    }

    if ordered_indexes.len() != sources.len() {
        let cycle = sources
            .iter()
            .enumerate()
            .filter_map(|(index, source)| (indegree[index] > 0).then_some(source.id.as_str()))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(anyhow!("manifest dependency cycle detected: {cycle}"));
    }

    Ok(ordered_indexes
        .into_iter()
        .map(|index| sources[index].clone())
        .collect())
}

fn resolve_dependency(
    dependency: &str,
    id_to_index: &HashMap<String, usize>,
    name_to_indexes: &HashMap<String, Vec<usize>>,
    source: &ManifestSource,
) -> Result<usize> {
    let normalized = dependency.trim().replace(':', "/");
    if let Some(index) = id_to_index.get(&normalized) {
        return Ok(*index);
    }

    if !normalized.contains('/') {
        if let Some(indexes) = name_to_indexes.get(&normalized) {
            if indexes.len() == 1 {
                return Ok(indexes[0]);
            }
            return Err(anyhow!(
                "manifest dependency '{}' referenced by '{}' is ambiguous; use kind/name",
                dependency,
                source.id
            ));
        }
    }

    Err(anyhow!(
        "manifest dependency '{}' referenced by '{}' was not found",
        dependency,
        source.id
    ))
}

fn manifest_id(path: &Path, document_index: usize, value: &Value, fallback_kind: &str) -> String {
    let kind = value
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or(fallback_kind);
    if let Some(name) = value
        .get("metadata")
        .and_then(|metadata| metadata.get("name"))
        .and_then(Value::as_str)
    {
        return format!("{kind}/{name}");
    }
    fallback_id(path, document_index, kind)
}

fn fallback_id(path: &Path, document_index: usize, kind: &str) -> String {
    format!("{}#doc{}:{kind}", path.display(), document_index + 1)
}

fn manifest_dependencies(value: &Value) -> Vec<String> {
    dependency_value(value)
        .map(parse_dependency_refs)
        .unwrap_or_default()
}

fn dependency_value(value: &Value) -> Option<&Value> {
    value
        .get("metadata")
        .and_then(|metadata| {
            metadata
                .get("dependsOn")
                .or_else(|| metadata.get("depends_on"))
        })
        .or_else(|| {
            value
                .get("spec")
                .and_then(|spec| spec.get("dependsOn").or_else(|| spec.get("depends_on")))
        })
        .or_else(|| value.get("dependsOn"))
        .or_else(|| value.get("depends_on"))
}

fn parse_dependency_refs(value: &Value) -> Vec<String> {
    match value {
        Value::String(dep) => vec![dep.trim().to_string()],
        Value::Array(items) => items
            .iter()
            .flat_map(parse_dependency_refs)
            .filter(|dep| !dep.is_empty())
            .collect(),
        Value::Object(map) => {
            if let Some(reference) = map
                .get("ref")
                .or_else(|| map.get("id"))
                .and_then(Value::as_str)
            {
                return vec![reference.trim().to_string()];
            }
            let kind = map.get("kind").and_then(Value::as_str);
            let name = map.get("name").and_then(Value::as_str).or_else(|| {
                map.get("metadata")
                    .and_then(|metadata| metadata.get("name"))
                    .and_then(Value::as_str)
            });
            match (kind, name) {
                (Some(kind), Some(name)) => vec![format!("{kind}/{name}")],
                (None, Some(name)) => vec![name.to_string()],
                _ => Vec::new(),
            }
        }
        _ => Vec::new(),
    }
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
    fn discover_orders_multi_document_manifests_by_dependency_dag() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join(".git")).unwrap();
        write(
            &tmp.path().join("tachyon.yml"),
            r#"actions:
  - context: fieldadmin
    name: Deploy
policies: []
metadata:
  name: fieldadmin-auth
  dependsOn:
    - CloudApps/fieldadmin
---
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: fieldadmin
spec:
  apps:
    - name: fieldadmin
"#,
        );

        let sources = discover(None, tmp.path()).unwrap();
        let ids = sources
            .iter()
            .map(|source| source.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            ids,
            vec!["CloudApps/fieldadmin", "AuthManifest/fieldadmin-auth"]
        );
    }

    #[test]
    fn discover_reports_missing_manifest_dependency() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join(".git")).unwrap();
        write(
            &tmp.path().join("tachyon.yml"),
            r#"apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: fieldadmin
  dependsOn:
    - OAuth2Client/missing
spec:
  apps:
    - name: fieldadmin
"#,
        );

        let error = discover(None, tmp.path()).unwrap_err().to_string();

        assert!(error.contains("OAuth2Client/missing"));
        assert!(error.contains("was not found"));
    }

    #[test]
    fn discover_reports_manifest_dependency_cycles() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join(".git")).unwrap();
        write(
            &tmp.path().join("tachyon.yml"),
            r#"apiVersion: apps.tachy.one/v1alpha
kind: Operator
metadata:
  name: platform
  dependsOn:
    - CloudApps/fieldadmin
---
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: fieldadmin
  dependsOn:
    - Operator/platform
spec:
  apps:
    - name: fieldadmin
"#,
        );

        let error = discover(None, tmp.path()).unwrap_err().to_string();

        assert!(error.contains("dependency cycle"));
        assert!(error.contains("Operator/platform"));
        assert!(error.contains("CloudApps/fieldadmin"));
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
