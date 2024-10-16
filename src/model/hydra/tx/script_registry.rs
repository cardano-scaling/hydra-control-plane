use pallas::txbuilder::Input;

pub struct ScriptRegistry {
    initial_reference: Input,
    commit_reference: Input,
    head_reference: Input,
}
