use polis_core::Result;

#[derive(Default)]
pub struct SandboxManager {
    // Sandbox management state
}

impl SandboxManager {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn create_sandbox(&self) -> Result<()> {
        // TODO: Implement sandbox creation
        Ok(())
    }
}
