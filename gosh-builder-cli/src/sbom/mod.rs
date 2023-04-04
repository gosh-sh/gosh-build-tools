#[derive(Debug, Default)]
pub struct Sbom {
    pub inner: Vec<String>,
}

impl Sbom {
    pub fn append(&mut self, record: String) {
        self.inner.push(record);
    }
}
