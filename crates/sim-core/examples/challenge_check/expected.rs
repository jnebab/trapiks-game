pub const SEARCH_RADIUS_M: f64 = 600.0;
const EDSA: &[&str] = &["edsa", "epifanio de los santos"];

pub fn expected_names(id: &str) -> &'static [&'static [&'static str]] {
    match id {
        "edsa-ortigas" => &[EDSA, &["ortigas"]],
        "magallanes" => &[EDSA, &["south luzon", "slex", "skyway", "osmeña", "osmena"]],
        "balintawak" => &[EDSA, &["north luzon", "nlex", "bonifacio"]],
        "edsa-quezon-ave" => &[EDSA, &["quezon avenue", "quezon ave"]],
        "c5-kalayaan" => &[
            &["c-5", "c5", "carlos p. garcia", "circumferential road 5"],
            &["kalayaan"],
        ],
        "espana-lacson" => &[&["españa", "espana"], &["lacson"]],
        "taft-buendia" => &[&["taft"], &["buendia", "puyat"]],
        _ => &[],
    }
}

pub fn names_match(expected: &[&[&str]], names: &[String]) -> bool {
    let lowered: Vec<String> = names.iter().map(|name| name.to_lowercase()).collect();
    expected.iter().all(|group| {
        group
            .iter()
            .any(|fragment| lowered.iter().any(|name| name.contains(fragment)))
    })
}
