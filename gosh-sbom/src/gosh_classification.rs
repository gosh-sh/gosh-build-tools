use cyclonedx_bom::models::component::Classification;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GoshClassification {
    File,
    Commit,
    Repository,
}

impl GoshClassification {
    pub fn to_component_type(&self) -> Classification {
        match self {
            GoshClassification::File => Classification::File,
            GoshClassification::Commit => Classification::Library,
            GoshClassification::Repository => Classification::Library,
        }
    }
}
