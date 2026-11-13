use floorplan::adapters::json::{render_json, render_plan_json};
use floorplan::domain::style::Style;
use floorplan::use_cases::generate::generate_house;



#[test]
fn json_carries_seed_and_style() {
    let json = render_json(&generate_house(42, Style::Ranch));
    assert!(json.contains("\"seed\":42"));
    assert!(json.contains("\"style\":\"ranch\""));
}

#[test]
fn json_has_rooms_and_openings() {
    let json = render_json(&generate_house(42, Style::Ranch));
    assert!(json.contains("\"program\":\"living\""));
    assert!(json.contains("\"kind\":\"door\""));
    assert!(json.contains("\"kind\":\"window\""));
}

#[test]
fn json_floats_have_two_decimals() {
    let json = render_json(&generate_house(42, Style::Ranch));
    assert!(json.contains("\"x\":0.00"), "first room should start at x 0.00");
}

#[test]
fn json_is_deterministic() {
    let house = generate_house(42, Style::Ranch);
    assert_eq!(render_json(&house), render_json(&house));
}

#[test]
fn json_lists_furniture_and_open_doors() {
    let json = render_json(&generate_house(42, Style::Modern));
    assert!(json.contains("\"furniture\":["), "the house JSON must list furniture");
    assert!(json.contains("\"kind\":\"bed\""), "a bed should be listed");
    assert!(json.contains("\"open_doors\":["), "the house JSON must list open doors");
}

#[test]
fn the_plan_json_stays_free_of_furniture() {
    // The variety hash is over the plan JSON; furniture is a
    // deterministic function of the plan, so it stays out.
    let json = render_plan_json(&generate_house(42, Style::Modern));
    assert!(!json.contains("furniture"), "the plan JSON must not carry furniture");
}
