use pallas::txbuilder::Input;

pub struct ScriptRegistry {
    pub initial_reference: Input,
    pub commit_reference: Input,
    pub head_reference: Input,
}
