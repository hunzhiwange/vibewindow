use super::*;

#[test]
fn resolve_provider_alias_normalizes_known_provider_id_case() {
    let ids = available_provider_ids();
    if let Some(first) = ids.first() {
        assert_eq!(resolve_provider_alias(&first.to_ascii_uppercase()), Some(first.clone()));
    }
    assert_eq!(resolve_provider_alias("unknown-provider"), None);
}

#[test]
fn available_provider_ids_is_sorted() {
    let ids = available_provider_ids();
    let mut sorted = ids.clone();
    sorted.sort();

    assert_eq!(ids, sorted);
}
