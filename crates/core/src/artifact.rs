use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactKind {
    Binary,
    Archive,
    Checksum,
    DockerImage,
    LinuxPackage,
    Metadata,
}

#[derive(Debug, Clone)]
pub struct Artifact {
    pub kind: ArtifactKind,
    pub path: PathBuf,
    pub target: Option<String>,
    pub crate_name: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Default)]
pub struct ArtifactRegistry {
    artifacts: Vec<Artifact>,
}

impl ArtifactRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, artifact: Artifact) {
        self.artifacts.push(artifact);
    }

    pub fn by_kind(&self, kind: ArtifactKind) -> Vec<&Artifact> {
        self.artifacts.iter().filter(|a| a.kind == kind).collect()
    }

    pub fn by_kind_and_crate(&self, kind: ArtifactKind, crate_name: &str) -> Vec<&Artifact> {
        self.artifacts
            .iter()
            .filter(|a| a.kind == kind && a.crate_name == crate_name)
            .collect()
    }

    pub fn all(&self) -> &[Artifact] {
        &self.artifacts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_add_and_query_artifacts() {
        let mut registry = ArtifactRegistry::new();
        registry.add(Artifact {
            kind: ArtifactKind::Binary,
            path: PathBuf::from("dist/cfgd"),
            target: Some("x86_64-unknown-linux-gnu".to_string()),
            crate_name: "cfgd".to_string(),
            metadata: Default::default(),
        });
        registry.add(Artifact {
            kind: ArtifactKind::Archive,
            path: PathBuf::from("dist/cfgd.tar.gz"),
            target: Some("x86_64-unknown-linux-gnu".to_string()),
            crate_name: "cfgd".to_string(),
            metadata: Default::default(),
        });

        let binaries = registry.by_kind(ArtifactKind::Binary);
        assert_eq!(binaries.len(), 1);

        let archives = registry.by_kind_and_crate(ArtifactKind::Archive, "cfgd");
        assert_eq!(archives.len(), 1);
    }

    #[test]
    fn test_empty_query() {
        let registry = ArtifactRegistry::new();
        assert!(registry.by_kind(ArtifactKind::Binary).is_empty());
    }
}
