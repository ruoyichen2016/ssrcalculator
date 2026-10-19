//! Renderer-independent blueprint geometry. Longitudinal coordinates run from
//! bow (0) to stern (1); renderers only project these normalized positions.
use crate::model::{GunBattery, ShipModel};
use crate::systems::CombatModality;

#[derive(Clone, Debug)]
pub struct BlueprintLayout {
    pub is_carrier: bool,
    pub batteries: [Vec<BlueprintMount>; 5],
    pub citadel: [f64; 2],
    pub superstructure: crate::superstructure::Plan,
    pub bridge: [f64; 2],
}

impl BlueprintLayout {
    pub fn from_ship(ship: &ShipModel) -> Self {
        let is_carrier =
            ship.systems.evaluate(ship).resolved_modality == CombatModality::AviationDeckProjection;
        let batteries = std::array::from_fn(|i| {
            solve_battery_mounts(
                &ship.guns.batteries[i],
                ship.free.fc_length,
                ship.free.fd_length,
                ship.free.qd_length,
                is_carrier,
                i != 0,
            )
        });
        let main = &batteries[0];
        let (lo, hi) = if main.is_empty() {
            (0.35, 0.65)
        } else {
            main.iter()
                .fold((1.0_f64, 0.0_f64), |(lo, hi), m| (lo.min(m.x), hi.max(m.x)))
        };
        let citadel = [(lo - 0.04).clamp(0.09, 0.38), (hi + 0.04).clamp(0.62, 0.91)];
        let superstructure = crate::superstructure::Plan::solve(ship, &batteries, is_carrier);
        let bridge_part = &superstructure.parts[0];
        let bridge = [bridge_part.x, bridge_part.x + bridge_part.length];
        Self {
            is_carrier,
            superstructure,
            batteries,
            citadel,
            bridge,
        }
    }
}

fn get_layout_pos(layout: i32, fc: f64, fd: f64, qd: f64) -> f64 {
    let ad = (1.0 - fc - fd - qd).max(0.0);
    match layout {
        0 | 9 => 0.5,
        1 | 10 => 0.38,
        2 | 11 => 0.62,
        3 | 12 => fc + 0.25 * fd,
        4 | 13 => fc + 0.50 * fd,
        5 | 14 => fc + 0.75 * fd,
        6 | 15 => 1.0 - (qd + 0.75 * ad),
        7 | 16 => 1.0 - (qd + 0.50 * ad),
        8 | 17 => 1.0 - (qd + 0.25 * ad),
        _ => 0.5,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BlueprintMount {
    pub x: f64,
    pub tier: i32,
    pub side: i32,
    pub fwd: bool,
    pub bpm: i32,
    pub is_casemate: bool,
    pub mount_type: i32,
}

#[allow(clippy::too_many_arguments)]
fn solve_group_mounts(
    count: i32,
    layout: i32,
    super_cnt: i32,
    bpm: i32,
    fc: f64,
    fd: f64,
    qd: f64,
    is_carrier: bool,
    is_sec: bool,
    mount_type: i32,
    is_sub: bool,
) -> Vec<BlueprintMount> {
    let mut out = Vec::new();
    if count <= 0 || !(0..18).contains(&layout) {
        return out;
    }
    let is_side = layout >= 9;
    let base_code = if is_side { layout - 9 } else { layout };
    let step = if is_sec { 0.045 } else { 0.062 };
    let bpm_clamped = bpm.max(1);
    let is_casemate = mount_type == 6 || mount_type == 0 || is_sub;
    let mut push_mount = |x: f64, tier: i32, side: i32, fwd: bool| {
        out.push(BlueprintMount {
            x: x.clamp(0.08, 0.92),
            tier,
            side,
            fwd,
            bpm: bpm_clamped,
            is_casemate,
            mount_type,
        });
    };

    if base_code == 1 || base_code == 2 {
        let n_fore = if base_code == 1 {
            (count + 1) / 2
        } else {
            count / 2
        };
        let n_aft = count - n_fore;
        let fore_start = if is_carrier {
            0.23
        } else {
            (fc * 0.52).clamp(0.14, 0.24)
        };
        let aft_end = if is_carrier {
            0.77
        } else {
            (1.0 - qd * 0.48).clamp(0.75, 0.86)
        };
        let mut s_rem = super_cnt.max(0);
        let f_super = if n_fore >= 2 && s_rem >= 1 { 1 } else { 0 };
        if f_super > 0 {
            s_rem -= 1;
        }
        let a_super = if n_aft >= 2 && s_rem >= 1 { 1 } else { 0 };

        for i in 0..n_fore {
            let tier = if i == 1 && f_super > 0 {
                1
            } else if i == 2 && n_fore >= 3 && super_cnt >= 3 {
                2
            } else {
                0
            };
            let side = if is_carrier {
                -1
            } else if is_side {
                if i % 2 == 0 {
                    -1
                } else {
                    1
                }
            } else {
                0
            };
            push_mount(fore_start + (i as f64) * step, tier, side, true);
        }
        for j in 0..n_aft {
            let tier = if j == 1 && a_super > 0 { 1 } else { 0 };
            let side = if is_carrier {
                -1
            } else if is_side {
                if j % 2 == 0 {
                    -1
                } else {
                    1
                }
            } else {
                0
            };
            push_mount(aft_end - (j as f64) * step, tier, side, false);
        }
    } else if base_code == 0 {
        if is_side {
            let pairs = (count + 1) / 2;
            let span = ((pairs - 1) as f64 * 0.07).clamp(0.12, 0.38);
            let start = 0.50 - span * 0.5;
            for i in 0..count {
                let pair_idx = i / 2;
                let x = if pairs <= 1 {
                    0.50
                } else {
                    start + (pair_idx as f64) * (span / ((pairs - 1).max(1) as f64))
                };
                let side = if i % 2 == 0 { -1 } else { 1 };
                push_mount(x, 0, side, x < 0.5);
            }
        } else {
            let start = 0.50 - ((count - 1) as f64) * step * 0.5;
            for i in 0..count {
                let x = start + (i as f64) * step;
                let tier = if i % 2 == 1 && super_cnt > 0 { 1 } else { 0 };
                push_mount(x, tier, 0, x < 0.5);
            }
        }
    } else {
        let center = get_layout_pos(layout, fc, fd, qd);
        let fwd = base_code <= 5;
        if is_side && !is_carrier {
            let pairs = (count + 1) / 2;
            for i in 0..count {
                let pair_idx = i / 2;
                let x = center + ((pair_idx as f64) - ((pairs - 1) as f64) * 0.5) * step;
                let tier = if super_cnt > 0 && pair_idx > 0 { 1 } else { 0 };
                let side = if i % 2 == 0 { -1 } else { 1 };
                push_mount(x, tier, side, fwd);
            }
        } else {
            for i in 0..count {
                let x = center + ((i as f64) - ((count - 1) as f64) * 0.5) * step;
                let tier = if super_cnt >= count || (i > 0 && i <= super_cnt) {
                    1
                } else {
                    0
                };
                let side = if is_carrier { -1 } else { 0 };
                push_mount(x, tier, side, fwd);
            }
        }
    }
    out
}

fn solve_battery_mounts(
    b: &GunBattery,
    fc: f64,
    fd: f64,
    qd: f64,
    is_carrier: bool,
    is_sec: bool,
) -> Vec<BlueprintMount> {
    if b.mounts <= 0 || b.calibre <= 0.0 {
        return Vec::new();
    }
    let mut list = Vec::new();
    let is_casemate_battery = b.mount_type == 6 || b.mount_type == 0;
    if b.mounts_g1 > 0 || b.mounts_g2 > 0 {
        if b.mounts_g1 > 0 {
            let is_sub1 =
                is_casemate_battery || (b.mount_sub_g1 > 0 && b.mount_sub_g1 >= b.mounts_g1);
            list.extend(solve_group_mounts(
                b.mounts_g1,
                b.layout_g1,
                b.mount_super_g1,
                b.per_mount_g1,
                fc,
                fd,
                qd,
                is_carrier,
                is_sec,
                b.mount_type,
                is_sub1,
            ));
        }
        if b.mounts_g2 > 0 {
            let is_sub2 =
                is_casemate_battery || (b.mount_sub_g2 > 0 && b.mount_sub_g2 >= b.mounts_g2);
            list.extend(solve_group_mounts(
                b.mounts_g2,
                b.layout_g2,
                b.mount_super_g2,
                b.per_mount_g2,
                fc,
                fd,
                qd,
                is_carrier,
                is_sec,
                b.mount_type,
                is_sub2,
            ));
        }
    } else {
        list = solve_group_mounts(
            b.mounts,
            b.layout_g1,
            b.mount_super_g1,
            b.per_mount_g1,
            fc,
            fd,
            qd,
            is_carrier,
            is_sec,
            b.mount_type,
            is_casemate_battery || b.mount_sub_g1 > 0,
        );
    }
    list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_handles_end_groups_casemates_carriers_and_absent_mounts() {
        let mut ship = crate::features::demo_ship();
        ship.systems.combat_modality = CombatModality::HeavyArtilleryCitadel;
        let b = &mut ship.guns.batteries[0];
        b.mounts = 4;
        b.mounts_g1 = 4;
        b.mounts_g2 = 0;
        b.layout_g1 = 1;
        b.mount_type = 3;
        b.mount_sub_g1 = 0;
        b.mount_super_g1 = 2;
        let layout = BlueprintLayout::from_ship(&ship);
        let mounts = &layout.batteries[0];
        assert_eq!(mounts.len(), 4);
        assert_eq!(mounts.iter().filter(|m| m.fwd).count(), 2);
        assert_eq!(mounts.iter().filter(|m| m.tier == 1).count(), 2);
        assert!(mounts.iter().all(|m| !m.is_casemate));
        assert!(mounts
            .iter()
            .all(|m| m.x >= layout.citadel[0] && m.x <= layout.citadel[1]));
        ship.guns.batteries[0].mount_type = 6;
        assert!(BlueprintLayout::from_ship(&ship).batteries[0]
            .iter()
            .all(|m| m.is_casemate));
        ship.systems.combat_modality = CombatModality::AviationDeckProjection;
        ship.ship_type.clear();
        let carrier = BlueprintLayout::from_ship(&ship);
        assert!(carrier.is_carrier);
        assert!(carrier.batteries[0].iter().all(|m| m.side == -1));
        ship.guns.batteries[0].layout_g1 = 18;
        assert!(BlueprintLayout::from_ship(&ship).batteries[0].is_empty());
    }
}
