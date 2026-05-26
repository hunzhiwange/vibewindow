use super::*;
use crate::identity::model::{AieosIdentity, IdentitySection, Names, PsychologySection};
use std::collections::HashMap;

#[test]
fn prompt_is_empty_for_empty_identity() {
    assert!(aieos_to_system_prompt(&AieosIdentity::default()).is_empty());
}

#[test]
fn prompt_sorts_neural_matrix_keys() {
    let mut matrix = HashMap::new();
    matrix.insert("zeta".to_string(), 0.2);
    matrix.insert("alpha".to_string(), 0.8);
    let identity = AieosIdentity {
        identity: Some(IdentitySection {
            names: Some(Names { full: Some("Nova".into()), ..Default::default() }),
            ..Default::default()
        }),
        psychology: Some(PsychologySection { neural_matrix: Some(matrix), ..Default::default() }),
        ..Default::default()
    };
    let prompt = aieos_to_system_prompt(&identity);
    assert!(prompt.contains("**姓名:** Nova"));
    assert!(prompt.find("alpha").unwrap() < prompt.find("zeta").unwrap());
}

