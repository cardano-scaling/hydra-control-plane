use super::input::InputWrapper;

#[derive(Clone)]
pub struct ScriptRegistry {
    pub initial_reference: InputWrapper,
    pub commit_reference: InputWrapper,
    pub head_reference: InputWrapper,
}
