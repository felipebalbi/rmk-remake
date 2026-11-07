//! Configuration-boundary regressions derived from REMAKE's KiCad nets.
use rmk_config::{resolved::hardware::BoardConfig, KeyboardTomlConfig};

fn config() -> KeyboardTomlConfig {
    KeyboardTomlConfig::new_from_toml_path(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../keyboard.toml"
    ))
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
fn reordered_thumbs_preserve_taps_holds_and_overrides_on_every_layer() {
    let map = config().keymap().unwrap();
    for layer in 0..4 {
        let wants = [
            "LT(1, Tab, THUMB)",
            "Enter",
            "Escape",
            "Delete",
            if layer >= 2 { "RCtrl" } else { "Space" },
            "LT(2, Backspace, THUMB)",
        ];
        for (col, want) in (2..=7).zip(wants) {
            // RMK searches down through transparent actions on active layers.
            let action = (0..=layer)
                .rev()
                .map(|l| map.keymap[l][3][col].as_str())
                .find(|action| *action != "_")
                .unwrap();
            assert_eq!(action, want, "layer {layer}, column {col}");
        }
    }
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
fn same_hand_thumbs_no_longer_trigger_escape_or_delete_combos() {
    let combos = config().behavior().unwrap().combos.unwrap().combos;
    assert_eq!(combos.len(), 2);
    for (combo, right_center) in combos.iter().zip(["Space", "RCtrl"]) {
        assert_eq!(combo.actions, ["Enter", right_center]);
        assert_eq!(combo.output, "CapsWordToggle");
    }
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
