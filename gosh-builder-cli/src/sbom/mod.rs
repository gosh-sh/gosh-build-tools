pub mod gosh_classification;

use cyclonedx_bom::models::tool::{Tool, Tools};
use cyclonedx_bom::prelude::{
    Bom, Component, Components, Metadata, NormalizedString, Purl, UrnUuid,
};
use gosh_classification::GoshClassification;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Default)]
pub struct Sbom {
    pub inner: Vec<(GoshClassification, String)>,
}

impl Sbom {
    pub fn append(&mut self, component_type: GoshClassification, raw_component: String) {
        self.inner.push((component_type, raw_component));
    }

    pub async fn save_to(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        // Note: Every BOM generated should have a unique serial number,
        // even if the contents of the BOM being generated have not changed
        // over time. The process or tool responsible for creating the BOM
        // should create random UUID's for every BOM generated.
        // TODO: fix temporary hack.
        let serial_number =
            UrnUuid::new("urn:uuid:3e671687-395b-41f5-a30f-a58921a69b79".to_string())
                .expect("Failed to create UrnUuid");
        let mut components = vec![];
        for (component_type, component) in &self.inner {
            let name = component;
            let version = "1.0.0";
            let bom_ref = None;
            let mut component =
                Component::new(component_type.to_component_type(), &name, version, bom_ref);
            component.purl = Some(Purl::new(
                "gosh",
                &component.name.to_string(),
                &component.version.to_string(),
            )?);
            components.push(component);
        }
        let bom = Bom {
            serial_number: Some(serial_number),
            metadata: Some(Metadata {
                tools: Some(Tools(vec![Tool {
                    name: Some(NormalizedString::new("gosh-docker-build")),
                    ..Tool::default()
                }])),
                ..Metadata::default()
            }),
            components: Some(Components(components)),
            ..Bom::default()
        };

        let mut output = Vec::<u8>::new();

        bom.output_as_json_v1_3(&mut output)
            .expect("Failed to write BOM");
        let mut sbom_file = File::create(path)?;
        sbom_file.write_all(&output)?;
        Ok(())
    }
}
