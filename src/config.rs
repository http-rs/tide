#[derive(Debug)]
pub(crate) enum Env {
    Dev,
    Prod,
    Test,
    Staging,
}

// NOTE: Example only!!
//
// Just to test if TypeMap works
pub(crate) struct Config {
    pub(crate) env: Env,
}
