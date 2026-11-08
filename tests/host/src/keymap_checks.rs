//! Configuration-boundary regressions derived from REMAKE's KiCad nets.
use rmk_config::{KeyboardTomlConfig, resolved::hardware::BoardConfig};

fn config() -> KeyboardTomlConfig {
    KeyboardTomlConfig::new_from_toml_path(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../keyboard.toml"
    ))
}

fn vial_definition() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../vial.json")).unwrap()
}

#[test]
fn vial_custom_keycode_slots_label_the_four_profile_rmk_user_commands() {
    let vial = vial_definition();
    let names: Vec<_> = vial["customKeycodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|key| key["name"].as_str().unwrap())
        .collect();
    // Vial assigns these positionally starting at QK_KB_0 / RMK User0.
    assert_eq!(
        names,
        [
            "BT0", "BT1", "BT2", "BT3", "NEXT_BT", "PREV_BT", "CLR_BT", "SWITCH"
        ]
    );
    assert_eq!(vial["lighting"], "none");
}

#[test]
fn vial_key_positions_match_the_36_switch_rmk_layout_without_phantom_thumbs() {
    let vial = vial_definition();
    assert_eq!(vial["matrix"]["rows"], 4);
    assert_eq!(vial["matrix"]["cols"], 10);
    let mut positions = Vec::new();
    for row in vial["layouts"]["keymap"].as_array().unwrap() {
        for key in row
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|key| key.as_str())
        {
            let (row, col) = key.split_once(',').unwrap();
            positions.push((row.parse::<u8>().unwrap(), col.parse::<u8>().unwrap()));
        }
    }
    assert_eq!(positions.len(), 36);
    positions.sort();
    assert!(positions.windows(2).all(|pair| pair[0] != pair[1]));
    let layout =
        rmk_config::layout::LayoutInfo::from_compressed_blob(&config().layout().unwrap().blob)
            .unwrap();
    for variant in layout.variants {
        let mut physical: Vec<_> = variant.keys.iter().map(|key| (key.row, key.col)).collect();
        physical.sort();
        assert_eq!(positions, physical);
    }
}

#[test]
fn vial_lock_has_a_reachable_physical_center_thumb_challenge() {
    let host = config().host();
    assert!(!host.insecure, "do not disable the physical-presence lock");
    assert_eq!(host.unlock_keys, [[3, 3], [3, 6]]);
}

#[test]
fn host_layout_exposes_only_the_36_physical_switches() {
    let layout =
        rmk_config::layout::LayoutInfo::from_compressed_blob(&config().layout().unwrap().blob)
            .unwrap();
    for variant in layout.variants {
        assert_eq!(variant.keys.len(), 36);
        let thumbs: Vec<_> = variant
            .keys
            .iter()
            .filter(|key| key.row == 3)
            .map(|key| key.col)
            .collect();
        assert_eq!(thumbs, [2, 3, 4, 5, 6, 7]);
    }
}

#[test]
fn miryoku_thumb_selectors_and_repeat_keys_resolve_at_the_physical_positions() {
    let map = config().keymap().unwrap();
    assert_eq!(map.keymap.len(), 7);
    let wants = [
        [
            "LT(3, Escape, THUMB)",
            "LT(1, Enter, THUMB)",
            "LT(2, Tab, THUMB)",
            "LT(5, Backspace, THUMB)",
            "LT(4, Space, THUMB)",
            "LT(6, Delete, THUMB)",
        ],
        ["No", "No", "No", "Backspace", "Space", "Delete"],
        ["No", "No", "No", "MouseBtn2", "MouseBtn1", "MouseBtn3"],
        ["No", "No", "No", "MediaStop", "MediaPlayPause", "AudioMute"],
        ["Dot", "0", "Minus", "No", "No", "No"],
        [
            "WM(9, LShift)",
            "WM(0, LShift)",
            "WM(Minus, LShift)",
            "No",
            "No",
            "No",
        ],
        ["Application", "Enter", "Tab", "No", "No", "No"],
    ];
    for (layer, wants) in wants.into_iter().enumerate() {
        for (col, want) in (2..=7).zip(wants) {
            assert_eq!(
                map.keymap[layer][3][col], want,
                "layer {layer}, column {col}"
            );
        }
    }
}

#[test]
fn layer_modifiers_stay_on_the_supporting_hand_in_the_existing_home_row_order() {
    let map = config().keymap().unwrap();
    assert_eq!(map.keymap.len(), 7);
    for layer in 1..=3 {
        assert_eq!(
            &map.keymap[layer][1][..5],
            ["LShift", "LAlt", "LGui", "LCtrl", "No"]
        );
    }
    for layer in 4..=6 {
        assert_eq!(
            &map.keymap[layer][1][5..],
            ["No", "RCtrl", "RGui", "RAlt", "RShift"]
        );
    }
    assert_eq!(
        &map.keymap[0][1],
        &[
            "MT(A, LShift)",
            "MT(S, LAlt)",
            "MT(D, LGui)",
            "MT(F, LCtrl)",
            "G",
            "H",
            "MT(J, RCtrl)",
            "MT(K, RGui)",
            "MT(L, RAlt)",
            "MT(Semicolon, RShift)",
        ]
    );
}

#[test]
fn four_profile_media_controls_use_the_profile_count_dependent_rmk_user_actions() {
    let config = config();
    assert_eq!(
        config
            .build_constants(&["nrf52840_ble", "split"])
            .unwrap()
            .ble_profiles_num,
        4
    );
    let map = config.keymap().unwrap();
    // With four profiles: User4=next, User5=previous, User6=clear, User7=USB/BLE.
    assert_eq!(
        &map.keymap[3][2][5..],
        ["User7", "User0", "User1", "User2", "User3"]
    );
    // REMAKE has no RGB LEDs or general-purpose external-power control.
    // Those Miryoku positions must remain disabled, not repurposed.
    assert_eq!(&map.keymap[3][0][5..], ["No"; 5]);
    assert_eq!(map.keymap[3][1][5], "No");
}

#[test]
fn absent_thumb_switches_have_no_action_on_any_layer() {
    let map = config().keymap().unwrap();
    for layer in &map.keymap {
        for col in [0, 1, 8, 9] {
            assert_eq!(layer[3][col], "No");
        }
    }
}

#[test]
fn caps_word_is_a_nav_key_without_competing_thumb_combos_or_tri_layer() {
    let config = config();
    let map = config.keymap().unwrap();
    assert_eq!(map.keymap[1][1][5], "CapsWordToggle");
    let behavior = config.behavior().unwrap();
    assert!(behavior.tri_layer.is_none());
    assert!(
        behavior
            .combos
            .is_none_or(|combos| combos.combos.is_empty())
    );
}

#[test]
fn physical_column_nets_reach_the_correct_logical_keys_on_both_halves() {
    let config = config();
    let BoardConfig::Split(split) = config.hardware().unwrap().board else {
        panic!("REMAKE must be split");
    };
    let map = config.keymap().unwrap();
    // JPL/JPR socket contacts traced to the vendor nice!nano v2 pinout.
    // U1's D18/P0.15 label is a typo: the actual module GPIO is P1.15.
    // The right module is flipped: socket n is module contact 25-n. The right
    // physical COL0 is the outer column (P), not the inner column (Y).
    for (board, nets, wants) in [
        (
            &split.central,
            ["P1_15", "P1_13", "P1_11", "P0_10", "P0_09"],
            ["Q", "W", "E", "R", "T"],
        ),
        (
            &split.peripheral[0],
            ["P1_15", "P1_13", "P1_11", "P0_10", "P0_09"],
            ["P", "O", "I", "U", "Y"],
        ),
    ] {
        let pins = board.matrix.col_pins.as_ref().unwrap();
        for (pin, want) in nets.into_iter().zip(wants) {
            let local_col = pins.iter().position(|p| p == pin).unwrap();
            assert_eq!(
                map.keymap[0][board.row_offset][board.col_offset + local_col],
                want
            );
        }
    }
}

#[test]
fn flipped_right_socket_routes_each_row_to_the_same_gpio_as_the_left() {
    let BoardConfig::Split(split) = config().hardware().unwrap().board else {
        panic!("REMAKE must be split");
    };
    for board in [&split.central, &split.peripheral[0]] {
        assert_eq!(
            board.matrix.row_pins.as_ref().unwrap(),
            &["P1_00", "P0_11", "P1_04", "P1_06"]
        );
        assert!(!board.matrix.row2col);
    }
}
