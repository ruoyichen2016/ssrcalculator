//! A Rust reimplementation of the SpringSharp design calculation engine.
//!
//! The original C# code lives in `raw/SpringSharpMono`.  This model ports the
//! naval architecture formulas used by SpringSharp while leaving out the
//! WinForms UI.  All dimensions/weights are in the same Imperial units used by
//! the original engine (feet, inches, long tons and pounds).

use std::fmt;

pub const FEET2METRES: f64 = 0.3048;
pub const INCH2MM: f64 = 25.4;
pub const POUNDS2KG: f64 = 0.45359236;
pub const HP2KW: f64 = 0.746;
pub const SQFEET2SQMETRES: f64 = 0.092903;
pub const POUND2TON: f64 = 2240.0;
pub const TON: f64 = 35.0;
pub const RANGE: f64 = 7000.0;
pub const INCH: f64 = 0.0185;

/// The original `Con` conversion enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Conversion {
    Feet2Metres,
    Inch2Mm,
    Hp2Kw,
    Pounds2Kg,
    SqFeet2SqMetres,
    None,
}

/// Equivalent of `MainForm.MetricCalc`.
pub fn metric_calc(imperial: f64, conversion: Conversion) -> f64 {
    match conversion {
        Conversion::Feet2Metres => imperial * FEET2METRES,
        Conversion::Inch2Mm => imperial * INCH2MM,
        Conversion::Hp2Kw => imperial * HP2KW,
        Conversion::Pounds2Kg => imperial * POUNDS2KG,
        Conversion::SqFeet2SqMetres => imperial * SQFEET2SQMETRES,
        Conversion::None => imperial,
    }
}

fn tc(x: f64) -> i32 {
    x as i32
}

fn pow(x: f64, y: f64) -> f64 {
    x.powf(y)
}

#[derive(Clone, Debug)]
pub struct GunBattery {
    // inputs
    pub barrels: i32,
    pub calibre: f64,
    pub cal_length: f64,
    pub date_gun: i32,
    pub gun_type: i32,
    pub mount_type: i32,
    pub magazine: i32,
    pub mounts: i32,
    pub guns_in_mount_g1: i32,
    pub guns_in_mount_g2: i32,
    pub layout_g1: i32,
    pub layout_g2: i32,
    pub mount_super_g1: i32,
    pub mount_sub_g1: i32,
    pub mount_super_g2: i32,
    pub mount_deck_g2: i32,
    pub mount_sub_g2: i32,
    pub super_super_chk: bool,
    pub sub_sub_chk: bool,
    pub arm_face: f64,
    pub arm_back: f64,
    pub arm_bottom: f64,
    pub weight_adj_g1: f64,
    pub weight_adj_g2: f64,

    // derived
    pub per_mount_g1: i32,
    pub per_mount_g2: i32,
    pub per_mount: i32,
    pub per_mounts: i32,
    pub gun_super_g1: i32,
    pub gun_sub_g1: i32,
    pub gun_super_g2: i32,
    pub gun_deck_g2: i32,
    pub gun_sub_g2: i32,
    pub mount_deck_g1: i32,
    pub gun_deck_g1: i32,
    pub mounts_g1: i32,
    pub mounts_g2: i32,
    pub mounts_for: f64,
    pub free_gun: f64,
    pub free_gun_g1: f64,
    pub free_gun_g2: f64,
    pub super_factor: f64,
    pub weight_adj: f64,
    pub mount_adj: f64,
    pub date_gun_adj: f64,
    pub shell_weight: f64,
    pub gun_weight: f64,
    pub mount_weight: f64,
    pub arm_weight: f64,
    pub arm_weight_face: f64,
    pub arm_weight_back: f64,
    pub arm_weight_bottom: f64,
    pub magazine_weight: f64,
    pub broadside_weight: f64,
    pub total_weight: f64,
    pub diameter_g1: f64,
    pub diameter_g2: f64,
    pub diameter_gun: f64,
    pub gun_house_height: f64,
    pub position_g1: f64,
    pub position_g2: f64,
    pub super_aft: bool,
}

impl Default for GunBattery {
    fn default() -> Self {
        Self {
            barrels: 0,
            calibre: 0.0,
            cal_length: 45.0,
            date_gun: 1950,
            gun_type: 1,
            mount_type: 5,
            magazine: 150,
            mounts: 0,
            guns_in_mount_g1: 0,
            guns_in_mount_g2: 0,
            layout_g1: 18,
            layout_g2: 18,
            mount_super_g1: 0,
            mount_sub_g1: 0,
            mount_super_g2: 0,
            mount_deck_g2: 0,
            mount_sub_g2: 0,
            super_super_chk: false,
            sub_sub_chk: false,
            arm_face: 0.0,
            arm_back: 0.0,
            arm_bottom: 0.0,
            weight_adj_g1: 1.0,
            weight_adj_g2: 1.0,
            per_mount_g1: 0,
            per_mount_g2: 0,
            per_mount: 0,
            per_mounts: 0,
            gun_super_g1: 0,
            gun_sub_g1: 0,
            gun_super_g2: 0,
            gun_deck_g2: 0,
            gun_sub_g2: 0,
            mount_deck_g1: 0,
            gun_deck_g1: 0,
            mounts_g1: 0,
            mounts_g2: 0,
            mounts_for: 0.0,
            free_gun: 0.0,
            free_gun_g1: 0.0,
            free_gun_g2: 0.0,
            super_factor: 1.0,
            weight_adj: 1.0,
            mount_adj: 0.5,
            date_gun_adj: 1.0,
            shell_weight: 0.0,
            gun_weight: 0.0,
            mount_weight: 0.0,
            arm_weight: 0.0,
            arm_weight_face: 0.0,
            arm_weight_back: 0.0,
            arm_weight_bottom: 0.0,
            magazine_weight: 0.0,
            broadside_weight: 0.0,
            total_weight: 0.0,
            diameter_g1: 0.0,
            diameter_g2: 0.0,
            diameter_gun: 0.0,
            gun_house_height: 0.0,
            position_g1: 0.0,
            position_g2: 0.0,
            super_aft: false,
        }
    }
}

impl GunBattery {
    pub fn per_mount_calc(index: i32) -> i32 {
        match index {
            0 => 1,
            1 | 3 | 4 => 2,
            2 | 5 | 9 | 10 => 4,
            6 | 7 => 3,
            8 => 6,
            11 => 8,
            12 | 13 => 5,
            14 => 10,
            _ => 0,
        }
    }

    fn shell_weight_calc(&mut self) -> f64 {
        let mut w = pow(self.calibre, 3.0) / 1.9830943211886;
        if self.cal_length < 45.0 {
            w *= (45.0 - (45.0_f64 - self.cal_length).sqrt()) / 45.0;
        } else if self.cal_length > 45.0 {
            w *= (45.0 + (self.cal_length - 45.0_f64).sqrt()) / 45.0;
        }
        w *= self.date_gun_adj;
        w
    }

    fn date_adj_calc(date: i32) -> f64 {
        if !(1850..=1950).contains(&date) {
            0.0
        } else if date <= 1890 {
            1.0 - (1890 - date) as f64 / 66.6666641
        } else {
            1.0
        }
    }

    fn free_gun_calc(&mut self, free: &FreeboardValues, layout: i32, mounts: i32) -> f64 {
        let fc_fd = free.fc_length + free.fd_length;
        self.mounts_for = if (layout == 0 && fc_fd >= 0.5 && mounts == 1)
            || (layout == 9 && fc_fd >= 0.5 && mounts == 1)
            || (layout == 1 && mounts == 1)
            || (layout == 10 && mounts == 1)
            || (layout > 2 && layout < 6)
            || (layout > 11 && layout < 15)
        {
            mounts as f64
        } else if (layout == 0 && fc_fd >= 0.5)
            || (layout == 9 && fc_fd >= 0.5)
            || layout == 1
            || layout == 10
        {
            (mounts as f64 * 0.5).round()
        } else if (layout == 0 && fc_fd < 0.5 && mounts == 1)
            || (layout == 9 && fc_fd < 0.5 && mounts == 1)
            || (layout == 2 && mounts == 1)
            || (layout == 11 && mounts == 1)
            || (layout > 5 && layout < 9)
            || layout > 14
        {
            0.0
        } else {
            mounts as f64 - (mounts as f64 * 0.5).round()
        };

        let mut fg = 0.0_f64;
        if mounts > 0 {
            match layout {
                0 | 9 => {
                    fg = (self.mounts_for * free.free_fd
                        + (mounts as f64 - self.mounts_for) * free.free_ad)
                        / mounts as f64;
                }
                1 | 2 | 10 | 11 => {
                    if self.mounts_for > 0.0 {
                        fg = self.mounts_for
                            * (((free.free_fd_for - free.free_fd) / self.mounts_for * 0.5)
                                + (free.free_fd_for + free.free_fd) * 0.5);
                    }
                    if mounts as f64 - self.mounts_for > 0.0 {
                        fg += (mounts as f64 - self.mounts_for)
                            * (((free.free_ad_aft - free.free_ad) * 1.0
                                / (mounts as f64 - self.mounts_for)
                                * 0.5)
                                + (free.free_ad_aft + free.free_ad) * 0.5);
                    }
                    fg /= mounts as f64;
                }
                3 | 12 => {
                    if self.mounts_for > 0.0 {
                        fg = ((free.free_fd_for - free.free_fd) / self.mounts_for * 0.5)
                            + (free.free_fd_for + free.free_fd) * 0.5;
                    }
                }
                4 | 13 => {
                    fg = free.free_fd;
                }
                5 | 14 => {
                    if self.mounts_for > 0.0 {
                        fg = ((free.free_fd_aft - free.free_fd) / self.mounts_for * 0.5)
                            + (free.free_fd_aft + free.free_fd) * 0.5;
                    }
                }
                6 | 15 => {
                    if mounts as f64 - self.mounts_for > 0.0 {
                        fg = (free.free_ad_for - free.free_ad) / (mounts as f64 - self.mounts_for)
                            * 0.5
                            + (free.free_ad_for + free.free_ad) * 0.5;
                    }
                }
                7 | 16 => {
                    fg = free.free_ad;
                }
                8 | 17 => {
                    if mounts as f64 - self.mounts_for > 0.0 {
                        fg = (free.free_ad_aft - free.free_ad) / (mounts as f64 - self.mounts_for)
                            * 0.5
                            + (free.free_ad_aft + free.free_ad) * 0.5;
                    }
                }
                _ => {}
            }
        }
        fg
    }

    fn layout_position(layout: i32, fd_length: f64, ad_length: f64, group: i32) -> f64 {
        match layout {
            3 => 0.25 * fd_length,
            4 => 0.5 * fd_length,
            5 => 0.75 * fd_length,
            6 => 0.25 * ad_length,
            7 => 0.5 * ad_length,
            8 => 0.75 * ad_length,
            12 => 0.25 * fd_length,
            13 => 0.5 * fd_length,
            14 => 0.75 * fd_length,
            15 => 0.25 * ad_length,
            16 => 0.5 * ad_length,
            17 => 0.75 * ad_length,
            _ => {
                if layout <= 2 || (9..=11).contains(&layout) || layout > 17 {
                    if group != 1 {
                        0.0
                    } else {
                        1.0
                    }
                } else {
                    0.0
                }
            }
        }
    }

    fn diameter_calc(mount_width: i32, calibre: f64, weight_adj: f64) -> f64 {
        let (num, y) = match mount_width {
            0 | 1 | 2 => (1.44, 0.609725),
            3 | 4 | 5 => (1.52, 0.4205),
            6 | 7 | 8 => (1.64, 0.29),
            9 | 10 | 11 => (1.8, 0.2),
            12 | 13 | 14 => (2.0, 0.14),
            _ => (0.0, 0.0),
        };
        let mut d = if calibre <= 0.0 {
            0.0
        } else {
            num * calibre * (1.0 + pow(1.0 / calibre, y))
        };
        if calibre < 12.0 {
            d += 12.0 / calibre;
        }
        if calibre > 1.0 && weight_adj < 1.0 {
            d *= 0.9;
        }
        d
    }

    /// Recalculate this battery's weights. `free` is the already-calculated
    /// freeboard model (needed for freeboard-dependent barbette heights).
    pub fn calculate(&mut self, free: &FreeboardValues) {
        self.date_gun_adj = Self::date_adj_calc(self.date_gun);
        self.per_mount_g1 = Self::per_mount_calc(self.guns_in_mount_g1);
        self.per_mount_g2 = Self::per_mount_calc(self.guns_in_mount_g2);

        // Split the mounts into the legacy G1/G2 groups.
        self.gun_super_g1 = self.mount_super_g1 * self.per_mount_g1;
        self.gun_sub_g1 = self.mount_sub_g1 * self.per_mount_g1;
        self.gun_super_g2 = self.mount_super_g2 * self.per_mount_g2;
        self.gun_deck_g2 = self.mount_deck_g2 * self.per_mount_g2;
        self.gun_sub_g2 = self.mount_sub_g2 * self.per_mount_g2;
        self.mount_deck_g1 = self.mounts
            - self.mount_super_g1
            - self.mount_sub_g1
            - self.mount_super_g2
            - self.mount_sub_g2
            - self.mount_deck_g2;
        self.gun_deck_g1 = self.barrels
            - self.gun_super_g1
            - self.gun_sub_g1
            - self.gun_super_g2
            - self.gun_deck_g2
            - self.gun_sub_g2;
        self.mounts_g1 = self.mount_super_g1 + self.mount_deck_g1 + self.mount_sub_g1;
        self.mounts_g2 = self.mount_super_g2 + self.mount_deck_g2 + self.mount_sub_g2;

        // The modern UI uses G1 only for most batteries, so derive the old G2
        // layout fields too for compatibility with the original code.
        self.free_gun_g1 = self.free_gun_calc(free, self.layout_g1, self.mounts_g1);
        self.free_gun_g2 = self.free_gun_calc(free, self.layout_g2, self.mounts_g2);
        if self.mounts_g1 > 0 || self.mounts_g2 > 0 {
            self.free_gun = (self.free_gun_g1 * self.mounts_g1 as f64
                + self.free_gun_g2 * self.mounts_g2 as f64)
                / (self.mounts_g1 + self.mounts_g2) as f64;
        } else {
            self.free_gun = 0.0;
        }

        self.mount_adj = match self.mount_type {
            0 => 0.5,
            1 => 1.0,
            2 => 0.7,
            3 => 1.0,
            4 => 1.0,
            5 => 0.5,
            6 => 0.5,
            _ => 1.0,
        };
        self.weight_adj = if self.mounts_g1 + self.mounts_g2 <= 0 {
            1.0
        } else {
            (self.weight_adj_g1 * self.mounts_g1 as f64
                + self.weight_adj_g2 * self.mounts_g2 as f64)
                / (self.mounts_g1 + self.mounts_g2) as f64
        };

        let gun_super_g2_adj = if self.super_super_chk { 2 } else { 1 };
        let gun_sub_g2_adj = if self.sub_sub_chk { 2 } else { 1 };
        if self.barrels > 0 {
            let s = (self.gun_super_g1 as f64 + (self.gun_super_g2 * gun_super_g2_adj) as f64
                - self.gun_sub_g1 as f64
                - (self.gun_sub_g2 * gun_sub_g2_adj) as f64)
                / self.barrels as f64;
            self.super_factor =
                (s * (7.5_f64).max(self.calibre * 0.6) + self.free_gun) / self.free_gun;
        } else {
            self.super_factor = 1.0;
        }

        self.shell_weight = self.shell_weight_calc();

        // Gun & mount weight (GunControl.GunWeightCalc)
        if self.calibre > 0.0 && self.barrels > 0 {
            self.gun_weight = self.shell_weight_calc()
                * (self.cal_length / 812.389434917877
                    * (1.0 + pow(1.0 / self.calibre, 2.3297949327695)))
                * self.barrels as f64;
        } else {
            self.gun_weight = 0.0;
        }

        if self.calibre > 1.0 {
            self.mount_weight = match self.mount_type {
                0 => 0.83,
                2 => 3.33,
                4 => 3.15,
                5 | 6 => 1.08,
                _ => 3.5,
            };
            match self.gun_type {
                0 => {
                    if self.mount_adj < 0.6 {
                        self.mount_weight *= 0.9;
                    } else {
                        self.mount_weight *= 0.98;
                    }
                }
                2 => {
                    if self.mount_adj < 0.6 {
                        self.mount_weight *= 1.35;
                    }
                }
                3 => {
                    if self.mount_adj < 0.6 {
                        self.mount_weight *= 1.44;
                    }
                }
                4 => {
                    if self.mount_adj < 0.6 {
                        self.mount_weight *= 1.57;
                    } else {
                        self.mount_weight *= 1.1;
                    }
                }
                5 => {
                    if self.mount_adj < 0.6 {
                        self.mount_weight *= 2.16;
                    } else {
                        self.mount_weight *= 1.5;
                    }
                }
                _ => {}
            }
            self.mount_weight += 1.0 / pow(self.calibre, 0.313068808543972);
            self.mount_weight *= self.gun_weight;
            if self.calibre > 10.0 {
                self.mount_weight *= 1.0 - 2.162376880645752 * self.calibre / 100.0;
            }
        } else {
            self.mount_weight = self.gun_weight;
        }
        self.mount_weight *= self.weight_adj;

        // Gun-house armour weight (GunControl.ArmWeightCalc)
        self.diameter_g1 =
            Self::diameter_calc(self.guns_in_mount_g1, self.calibre, self.weight_adj_g1);
        self.diameter_g2 =
            Self::diameter_calc(self.guns_in_mount_g2, self.calibre, self.weight_adj_g2);
        self.gun_house_height = if self.mount_type != 1 {
            (7.5_f64).max(0.625 * self.calibre)
        } else {
            (7.5_f64).max(0.625 * self.calibre * 2.0)
        };
        if self.mounts_g1 == 0 {
            self.per_mount_g1 = 0;
        }
        if self.mounts_g2 == 0 {
            self.per_mount_g2 = 0;
        }

        if self.calibre > 0.0 {
            self.arm_weight_face = if self.mount_type == 0 || self.mount_type == 6 {
                1.0
            } else if self.mount_type == 1 {
                std::f64::consts::FRAC_PI_2
            } else if self.mount_type == 2 {
                0.0
            } else if self.arm_back != 0.0 {
                0.5
            } else {
                1.5
            };
            self.arm_weight_face *= (self.diameter_g1 * self.mounts_g1 as f64
                + self.diameter_g2 * self.mounts_g2 as f64)
                * self.gun_house_height
                * self.arm_face
                * INCH;
            if self.gun_type == 3 || (self.gun_type == 6 && self.arm_back == 0.0) {
                self.arm_weight_face *= 0.333;
            }

            self.arm_weight_back = if self.mount_type <= 2 || self.mount_type >= 6 {
                0.0
            } else {
                2.5
            };
            let ring_area = |d: f64, n: i32| std::f64::consts::PI * pow(d / 2.0, 2.0) * n as f64;
            if self.mount_type == 1 {
                self.arm_weight_back = ring_area(self.diameter_g1, self.mounts_g1)
                    + ring_area(self.diameter_g2, self.mounts_g2);
            } else {
                self.arm_weight_back *= (self.diameter_g1 * self.mounts_g1 as f64
                    + self.diameter_g2 * self.mounts_g2 as f64)
                    * self.gun_house_height;
                self.arm_weight_back += (ring_area(self.diameter_g1, self.mounts_g1)
                    + ring_area(self.diameter_g2, self.mounts_g2))
                    * 0.75;
            }
            self.arm_weight_back *= self.arm_back * INCH;

            if self.free_gun > 0.0 {
                if self.mounts_g1 + self.mounts_g2 > 0 {
                    self.per_mounts = (self.per_mount_g1 * self.mounts_g1
                        + self.per_mount_g2 * self.mounts_g2)
                        / (self.mounts_g1 + self.mounts_g2);
                    if self.mount_adj > 0.5 {
                        self.per_mounts = self.per_mounts.min(4);
                    } else {
                        // This odd-looking branch mirrors the decompiled C#:
                        // the assignment goes to per_mount_g1, not per_mounts.
                        self.per_mount_g1 = self.per_mounts.min(5);
                    }
                }
                self.arm_weight_bottom = match self.mount_type {
                    2 => 0.6416,
                    3 => 0.5,
                    4 | 6 => 0.1,
                    _ => 0.0,
                };
                self.arm_weight_bottom = (1.0 - (self.per_mounts as f64 - 2.0) / 6.0)
                    * self.arm_bottom
                    * self.barrels as f64
                    * pow(self.calibre, 1.2)
                    * self.arm_weight_bottom
                    * self.free_gun
                    / 16.0
                    * self.super_factor
                    * self.arm_weight_bottom
                    * 2.0
                    * self.date_gun_adj.sqrt();
            } else {
                self.arm_weight_bottom = 0.0;
            }
        } else {
            self.arm_weight_face = 0.0;
            self.arm_weight_back = 0.0;
            self.arm_weight_bottom = 0.0;
        }

        self.arm_weight = self.arm_weight_face + self.arm_weight_back + self.arm_weight_bottom;
        self.total_weight = self.gun_weight + self.mount_weight + self.arm_weight;
        self.magazine_weight = (self.magazine as f64 * self.barrels as f64 * self.shell_weight
            / POUND2TON)
            * 1.2444443702697754;
        self.broadside_weight = self.barrels as f64 * self.shell_weight;

        // Layout positions used by the longitudinal strength calculation.
        self.position_g1 = Self::layout_position(self.layout_g1, free.fd_length, free.ad_length, 1);
        self.position_g2 = Self::layout_position(self.layout_g2, free.fd_length, free.ad_length, 2);
        self.super_aft = false;
    }
}

/// Freeboard inputs plus the values derived from them.
#[derive(Clone, Debug, Default)]
pub struct FreeboardValues {
    pub fc_length: f64,
    pub fd_length: f64,
    pub qd_length: f64,
    pub free_fc_for: f64,
    pub free_fc_aft: f64,
    pub free_fd_for: f64,
    pub free_fd_aft: f64,
    pub free_ad_for: f64,
    pub free_ad_aft: f64,
    pub free_qd_for: f64,
    pub free_qd_aft: f64,
    pub length_bow: f64,
    pub length_ram: f64,
    pub length_stern: f64,
    pub bow: i32,
    pub stern: i32,
    pub transom_width: f64,

    // derived
    pub ad_length: f64,
    pub free_fc: f64,
    pub free_fd: f64,
    pub free_ad: f64,
    pub free_qd: f64,
    pub freeboard: f64,
    pub free_distributed: f64,
    pub freeboard_cap: f64,
    pub length_stem: f64,
}

impl FreeboardValues {
    pub fn calculate(&mut self) {
        self.ad_length = 1.0 - self.fc_length - self.fd_length - self.qd_length;
        self.free_fc = self.free_fc_aft + (self.free_fc_for - self.free_fc_aft) * 0.4;
        self.free_fd = self.free_fd_for + (self.free_fd_aft - self.free_fd_for) * 0.5;
        self.free_ad = self.free_ad_for + (self.free_ad_aft - self.free_ad_for) * 0.5;
        self.free_qd = self.free_qd_for + (self.free_qd_aft - self.free_qd_for) * 0.5;
        self.freeboard = self.free_fc * self.fc_length
            + self.free_fd * self.fd_length
            + self.free_ad * self.ad_length
            + self.free_qd * self.qd_length;
        let denom = self.fd_length + self.ad_length;
        self.free_distributed = if denom.abs() > f64::EPSILON {
            (self.free_fd * self.fd_length + self.free_ad * self.ad_length) / denom
        } else {
            0.0
        };
    }

    /// Compute the capped freeboard used for strength/seakeeping calculations.
    /// Needs the battery data (low submerged guns reduce usable freeboard).
    pub fn calculate_cap(&mut self, beam: f64, guns: &[GunBattery; 5]) {
        let has_submerged = guns
            .iter()
            .any(|g| g.mount_type == 0 && g.gun_sub_g1 + g.gun_sub_g2 > 0);
        self.freeboard_cap = if self.freeboard <= beam / 3.0 {
            if has_submerged {
                self.freeboard - 6.0
            } else {
                self.freeboard
            }
        } else {
            self.freeboard * self.freeboard / (beam / 3.0)
        };
    }
}

/// One small weapon group (torpedo, mine or depth charge/AS mortar).
#[derive(Clone, Debug, Default)]
pub struct WeaponMount {
    pub kind: WeaponKind,
    pub number: i32,
    pub loads: i32,
    pub size: f64,
    pub length: f64,
    pub location: i32,

    // derived
    pub area: f64,
    pub weapon_weight: f64,
    pub mount_weight: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WeaponKind {
    #[default]
    Torpedo,
    Mine,
    DepthCharge,
}

impl WeaponMount {
    pub fn calculate(&mut self, date_hull: i32, beam: f64) {
        if self.kind == WeaponKind::Torpedo {
            if self.loads == 0 {
                self.loads = 1;
            }
            let n = self.number;
            let l = self.loads;
            let per_set = n / l;
            match self.location {
                0 => {
                    self.area = self.length * self.size / 12.0 * n as f64;
                }
                1 => {
                    let width = (per_set as f64) * self.size / 12.0 + (per_set as f64 - 1.0) * 0.5;
                    let hyp = (self.length * self.length + width * width).sqrt();
                    self.area =
                        pow(hyp * 0.5, 2.0) * std::f64::consts::PI + width * 0.5 * self.length;
                }
                2 => {
                    let width = (per_set as f64) * self.size / 12.0 + (per_set as f64 - 1.0) * 0.5;
                    self.area =
                        (self.length * self.length + width * width).sqrt() * beam * l as f64;
                }
                3 => {
                    self.area = self.length * 1.5 * (self.size + 6.0) / 12.0 * n as f64;
                }
                4 | 5 | 6 | 7 => {
                    self.area = self.length * 2.5 * pow(self.size * 2.75 / 12.0, 2.0) * n as f64;
                }
                8 => {
                    self.area = self.length * 1.5 * pow(self.size * 1.5 / 12.0, 2.0) * n as f64;
                }
                _ => self.area = 0.0,
            }
            if self.size > 0.0 && self.length > 0.0 && n > 0 {
                let mut f = if date_hull < 1907 {
                    (1907 - date_hull) as f64
                } else {
                    0.0
                };
                f = (f + 25.0) * 937.0;
                self.weapon_weight = (std::f64::consts::PI * self.size * self.size * self.length)
                    / f
                    + 0.0040000001899898052 * (date_hull - 1890) as f64;
                self.weapon_weight *= n as f64;
            } else {
                self.weapon_weight = 0.0;
            }
            self.mount_weight = match self.location {
                0 | 3 | 8 => 0.25,
                1 | 2 | 4 | 5 | 6 | 7 => 1.0,
                _ => 0.0,
            } * self.weapon_weight;
        } else {
            self.area = 0.0;
            self.weapon_weight = (self.number + self.loads) as f64 * self.size / POUND2TON;
            self.mount_weight = if self.kind == WeaponKind::Mine {
                match self.location {
                    0 => 0.25,
                    1 | 2 | 3 => 1.0,
                    _ => 0.0,
                }
            } else {
                match self.location {
                    0 => 0.25,
                    1 | 2 => 0.5,
                    3 => 10.0,
                    _ => 0.0,
                }
            } * self.weapon_weight;
        }
    }
}

/// Weight of all secondary weapons plus user-entered miscellaneous weights.
#[derive(Clone, Debug, Default)]
pub struct WeaponAggregate {
    pub torpedo1: WeaponMount,
    pub torpedo2: WeaponMount,
    pub mine: WeaponMount,
    pub dc1: WeaponMount,
    pub dc2: WeaponMount,
    pub misc_weight_vital: f64,
    pub misc_weight_hull: f64,
    pub misc_weight_deck: f64,
    pub misc_weight_above: f64,
    pub void_weight: f64,
    pub space_deck: f64,
    pub space_hull: f64,
    pub weight: f64,
}

impl WeaponAggregate {
    pub fn calculate(&mut self, date_hull: i32, hull: &HullValues) {
        let beam = hull.beam;
        self.torpedo1.calculate(date_hull, beam);
        self.torpedo2.calculate(date_hull, beam);
        self.mine.calculate(date_hull, beam);
        self.dc1.calculate(date_hull, beam);
        self.dc2.calculate(date_hull, beam);

        self.space_deck = 0.0;
        if self.torpedo1.location < 4 {
            self.space_deck = self.torpedo1.area;
        }
        if self.torpedo2.location < 4 {
            self.space_deck += self.torpedo2.area;
        }
        if hull.waterplane_area.abs() > f64::EPSILON {
            self.space_deck /= hull.waterplane_area;
        }

        self.space_hull = 0.0;
        if self.torpedo1.location > 3 {
            self.space_hull = self.torpedo1.area;
        }
        if self.torpedo2.location > 3 {
            self.space_hull += self.torpedo2.area;
        }
        let hull_volume = hull.length * hull.beam_bulges * hull.draught * hull.block;
        if hull_volume.abs() > f64::EPSILON {
            self.space_hull /= hull_volume;
        }

        self.weight = self.torpedo1.weapon_weight
            + self.torpedo1.mount_weight
            + self.torpedo2.weapon_weight
            + self.torpedo2.mount_weight
            + self.mine.weapon_weight
            + self.mine.mount_weight
            + self.dc1.weapon_weight
            + self.dc1.mount_weight
            + self.dc2.weapon_weight
            + self.dc2.mount_weight;
    }
}

/// Hull derived values.
#[derive(Clone, Debug, Default)]
pub struct HullValues {
    // inputs
    pub length: f64,
    pub length_oa: f64,
    pub beam: f64,
    pub beam_bulges: f64,
    pub draught: f64,
    pub block: f64,
    pub date_hull: i32,
    pub date_engine: i32,
    pub screws: i32,
    pub transom_width: f64,
    pub stern: i32,
    // Optional enhanced resistance/hull-form parameters (imperial units).
    pub lcb_percent: f64,
    pub midship_coefficient: f64,
    pub bulb_area_sqft: f64,
    pub bulb_centre_height_ft: f64,
    pub transom_area_sqft: f64,
    pub appendage_area_sqft: f64,
    pub stern_shape: f64,

    // derived
    pub displacement: f64,
    pub waterplane_area: f64,
    pub wetted_surface: f64,
    pub co_water_plane: f64,
    pub co_sharp: f64,
    pub length_beam: f64,
    pub length_effect: f64,
    pub speed_natural: f64,
    pub crew_max: i32,
    pub crew_min: i32,
    pub length_stem: f64,
    pub draught_deep: f64,
    pub draught_side: f64,
}

impl HullValues {
    pub fn calculate(&mut self, _free: &FreeboardValues) {
        self.displacement = self.length * self.beam_bulges * self.draught * self.block / TON;
        self.wetted_surface = self.length * self.draught * 1.7
            + self.displacement * TON / self.draught.max(f64::EPSILON);

        let max_block = self.block.max(0.4);
        self.co_water_plane = 0.78799998760223389 - (1.0 - max_block) * 0.13570000231266022;
        if self.screws > 1 {
            self.co_water_plane = (0.079999998211860657 * self.transom_width + 0.75999999046325684)
                .max(self.co_water_plane);
        }
        self.co_water_plane = (0.26199999451637268
            + self.co_water_plane
                * (max_block
                    / (1.00600004196167
                        - 0.00559999980032444 * pow(max_block, -3.559999942779541))))
        .min(1.0);
        if self.block < 0.4 {
            self.co_water_plane -=
                0.0281 - pow(self.block - 0.30000001192092896, 1.5499999523162842);
        }
        self.waterplane_area = self.co_water_plane * self.length * self.beam;

        self.co_sharp =
            0.4 * pow(
                self.beam_bulges / self.length.max(f64::EPSILON) * 6.0,
                0.3333333432674408,
            ) * (self.block / 0.51999998092651367).sqrt();
        self.length_beam = self.length / self.beam_bulges.max(f64::EPSILON);
        self.length_effect = if self.stern != 1 {
            self.length
        } else {
            self.beam_bulges * pow(self.transom_width.min(0.6666667), 1.7100000381469727)
                / 0.41749998927116394
                / self.co_sharp
                + self.length
        };
        self.speed_natural = self.length_effect.max(0.0).sqrt();
        self.crew_max = tc(pow(self.displacement, 0.75) * 0.65);
        self.crew_min = tc(self.crew_max as f64 * 0.7692);
        self.draught_side = ((1.006 - 0.0056 * pow(self.block, -3.56)) * 2.0 - 1.0) * self.draught;
    }
}

#[derive(Clone, Debug, Default)]
pub struct ArmourValues {
    pub belt_main: f64,
    pub belt_main_length: f64,
    pub belt_main_height: f64,
    pub belt_ends: f64,
    pub belt_ends_length: f64,
    pub belt_ends_height: f64,
    pub belt_upper: f64,
    pub belt_upper_length: f64,
    pub belt_upper_height: f64,
    pub belts_incline: f64,
    pub bulkhead: f64,
    pub bulkhead_length: f64,
    pub bulkhead_height: f64,
    pub bulkhead_beam: f64,
    pub bulkhead_type: i32,
    pub bulge: f64,
    pub bulge_length: f64,
    pub bulge_height: f64,
    pub deck: f64,
    pub deck_fc: f64,
    pub deck_qd: f64,
    pub deck_coverage: i32,
    pub conning_for: f64,
    pub conning_aft: f64,

    pub weight_belt_main: f64,
    pub weight_belt_ends: f64,
    pub weight_belt_upper: f64,
    pub weight_belts: f64,
    pub weight_bulkhead: f64,
    pub weight_bulge: f64,
    pub weight_deck: f64,
    pub weight_conning: f64,
    pub weight: f64,
    pub max_belt_height: f64,
    pub belt_cover: f64,
}

impl ArmourValues {
    /// Recompute armour weights. `hull`, `free`, `guns` and `engine_weight`
    /// are needed by the deck armour coverage options.
    #[allow(clippy::too_many_arguments)]
    pub fn calculate(
        &mut self,
        hull: &HullValues,
        free: &FreeboardValues,
        guns: &GunsAggregate,
        engine_weight: f64,
    ) {
        // Main side belt
        self.weight_belt_main = (self.belt_main_length
            + pow(
                (hull.length - self.belt_main_length) / hull.length.max(f64::EPSILON),
                1.0 - hull.co_water_plane,
            ) * hull.beam)
            * self.belt_main_height
            * self.belt_main
            * INCH
            * 2.0;
        self.weight_belt_ends =
            2.0 * self.belt_ends_length * self.belt_ends_height * self.belt_ends * INCH;
        self.weight_belt_upper = (self.belt_upper_length
            + pow(
                (hull.length - self.belt_upper_length) / hull.length.max(f64::EPSILON),
                1.0 - hull.co_water_plane,
            ) * hull.beam)
            * self.belt_upper_height
            * self.belt_upper
            * INCH
            * 2.0;
        self.weight_belts = self.weight_belt_main + self.weight_belt_ends + self.weight_belt_upper;
        self.max_belt_height = (hull.draught + free.free_distributed)
            * (1.0 / (self.belts_incline.abs() * std::f64::consts::PI / 180.0).cos())
            + 0.02;
        self.weight_bulkhead =
            2.0 * self.bulkhead_length * self.bulkhead_height * self.bulkhead * INCH;
        self.weight_bulge = 2.0 * self.bulge_length * self.bulge_height * self.bulge * INCH;

        // Deck armour: several coverage modes depend on machinery/magazine.
        self.weight_deck = match self.deck_coverage {
            0 | 1 | 2 | 3 => {
                let w = hull.waterplane_area
                    - pow(free.fc_length * 2.0, 1.0 - pow(hull.co_water_plane, 2.0))
                        * hull.beam
                        * hull.length
                        * free.fc_length
                        / 2.0
                    - (pow(free.qd_length, 1.0 - hull.co_water_plane)
                        * hull.beam
                        * hull.length
                        * free.qd_length
                        / 4.0
                        + (pow(free.qd_length, 1.0 - hull.co_water_plane)
                            + pow(free.qd_length * 2.0, 1.0 - hull.co_water_plane))
                            * hull.beam
                            * hull.length
                            * free.qd_length
                            / 4.0);
                w * 1.01
            }
            4 => {
                (engine_weight * 3.0 / (hull.displacement * 0.93999999761581421)
                    * 0.64999997615814209
                    * hull.length
                    + 16.0)
                    * (hull.beam + 16.0)
                    - 256.0
            }
            5 => {
                (guns.magazine / (hull.displacement * 0.93999999761581421)
                    * 0.64999997615814209
                    * hull.length
                    + 16.0)
                    * (hull.beam + 16.0)
                    - 256.0
            }
            6 => {
                ((engine_weight * 3.0 + guns.magazine) / (hull.displacement * 0.93999999761581421)
                    * 0.64999997615814209
                    * hull.length
                    + 16.0)
                    * (hull.beam + 16.0)
                    - 256.0
            }
            _ => 0.0,
        };
        self.weight_deck *= self.deck;
        self.weight_deck += pow(free.fc_length * 2.0, 1.0 - pow(hull.co_water_plane, 2.0))
            * hull.beam
            * hull.length
            * free.fc_length
            / 2.0
            * self.deck_fc;
        self.weight_deck += (pow(free.qd_length, 1.0 - hull.co_water_plane)
            * hull.beam
            * hull.length
            * free.qd_length
            / 4.0
            + (pow(free.qd_length, 1.0 - hull.co_water_plane)
                + pow(free.qd_length * 2.0, 1.0 - hull.co_water_plane))
                * hull.beam
                * hull.length
                * free.qd_length
                / 4.0)
            * self.deck_qd;
        self.weight_deck *= INCH;

        self.weight_conning = 10.0
            * pow(hull.displacement / 10000.0, 0.66666668653488159)
            * (self.conning_for + self.conning_aft);
        self.belt_cover = self.belt_main_length / hull.length / 0.64999997615814209;
        self.weight = self.weight_belts
            + self.weight_bulkhead
            + self.weight_bulge
            + self.weight_deck
            + self.weight_conning
            + guns.arm_weight;
    }
}

/// Aggregate gun weights used by the rest of the engine.
#[derive(Clone, Debug, Default)]
pub struct GunsAggregate {
    pub batteries: [GunBattery; 5],
    pub arm_weight: f64,
    pub broadside: f64,
    pub borne_weight: f64,
    pub guns_weight: f64,
    pub magazine: f64,
    pub super_factor: f64,
    pub unfixed_weight: f64,
}

impl GunsAggregate {
    pub fn calculate(&mut self, free: &FreeboardValues) {
        for g in self.batteries.iter_mut() {
            g.calculate(free);
        }
        self.unfixed_weight = self.batteries.iter().map(|g| g.gun_weight).sum();
        self.arm_weight = self.batteries.iter().map(|g| g.arm_weight).sum();
        self.magazine = self.batteries.iter().map(|g| g.magazine_weight).sum();
        self.broadside = self.batteries.iter().map(|g| g.broadside_weight).sum();
        self.guns_weight =
            self.unfixed_weight + self.batteries.iter().map(|g| g.mount_weight).sum::<f64>();
        self.borne_weight = self
            .batteries
            .iter()
            .map(|g| g.gun_weight * g.mount_adj)
            .sum::<f64>()
            * 2.0;
        if self.arm_weight + self.guns_weight > 0.0 {
            self.super_factor = self
                .batteries
                .iter()
                .map(|g| g.total_weight * g.super_factor * g.mount_adj)
                .sum::<f64>()
                / (self.arm_weight + self.guns_weight);
        } else {
            self.super_factor = 1.0;
        }
    }
}

/// Resistance model used by the engine.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResistanceModel {
    /// The original compact SpringSharp empirical resistance model.
    #[default]
    Original,
    /// Holtrop-Mennen style resistance/power prediction.
    HoltropMennen,
}

/// Machinery and displacement calculations.
#[derive(Clone, Debug, Default)]
pub struct EngineValues {
    // inputs
    pub fuel: [bool; 5],
    pub engine_types: [bool; 3],
    pub drive: [bool; 4],
    pub speed_max: f64,
    pub speed_cruise: f64,
    pub range: f64,
    pub pct_coal: f64,
    pub screws: i32,
    pub power_factor: f64,
    /// Calibration factor applied to the original SpringSharp empirical model
    /// (used when blending or when `resistance_model = original`).
    pub original_power_factor: f64,
    /// 1.0 = use the selected `resistance_model` only.  A value in (0,1) blends
    /// the Holtrop-Mennen result (weight `blend_weight`) with the original
    /// SpringSharp result (weight `1 - blend_weight`).
    pub blend_weight: f64,
    pub resistance_model: ResistanceModel,
    pub holtrop_viscous_factor: f64,
    pub holtrop_wave_factor: f64,
    pub holtrop_correlation_factor: f64,
    pub planing_calibration_factor: f64,
    pub cfd_proxy_enabled: bool,
    pub cfd_proxy_viscous: f64,
    pub cfd_proxy_wave: f64,
    pub cfd_proxy_fn_ref: f64,
    pub planing_min_speed_kt: f64,
    pub planing_max_displacement_tons: f64,
    pub ensemble_reduced_weight: f64,
    pub ensemble_structural_weight: f64,
    /// Multiplier on the empirical engine weight (1.0 = standard capital-ship steam baseline).
    /// Used for lightweight/compact machinery (e.g. WWII high-pressure destroyer steam turbines ~0.50).
    pub machinery_weight_factor: f64,
    pub engine_lock: bool,

    // derived resistance
    pub frict_max: f64,
    pub frict_cruise: f64,
    pub wave_max: f64,
    pub wave_cruise: f64,
    pub power_wave_max: f64,
    pub power_wave_cruise: f64,
    pub hp_max: f64,
    pub hp_cruise: f64,
    pub length_hp: f64,
    pub speed_hp: f64,
    pub wake_fraction: f64,
    pub thrust_deduction: f64,
    pub hull_efficiency: f64,
    pub open_water_efficiency: f64,
    pub relative_rotative_efficiency: f64,
    pub shaft_efficiency: f64,
    pub qpc: f64,
    /// True only when the QPC/efficiency fields participate in the power
    /// calculation (Holtrop-Mennen).  The original SpringSharp empirical fit
    /// has no explicit propulsive coefficient, so the breakdown does not
    /// apply to it.
    pub qpc_applicable: bool,

    // derived machinery
    pub engine: f64,
    pub engine_recip_simple: f64,
    pub engine_recip_complex: f64,
    pub engine_turbine: f64,
    pub engine_count: i32,
    pub engine_weight: f64,
    pub disp_factor: f64,
    pub bunker: f64,
    pub load_weight: f64,
    pub hull_weight: f64,
}

impl EngineValues {
    pub fn speed_hp_calc(&mut self, speed: f64, hull: &HullValues) -> f64 {
        self.length_hp = if speed > 15.0 {
            if speed < 25.0 {
                (hull.length_effect - hull.length) * ((speed - 20.0) / 5.0) + hull.length
            } else {
                hull.length_effect
            }
        } else {
            hull.length - (hull.length_effect - hull.length)
        };
        let mut original_hp = (pow(hull.displacement, 0.66666668653488159) / self.length_hp
            * hull.co_sharp
            * pow(speed, 4.0)
            + 0.01 * hull.wetted_surface * pow(speed, 1.83))
            * speed
            / 184.1666667;
        if hull.date_engine < 1890 {
            original_hp *= 1.0 + (1890.0 - hull.date_engine as f64) / 100.0;
        }
        original_hp *= self.original_power_factor;

        let fn_ = crate::cfd_proxy::cfd_froude_number(speed, hull.length);
        let cfd = crate::cfd_proxy::CfdProxyConfig {
            enabled: self.cfd_proxy_enabled,
            viscous_correction: self.cfd_proxy_viscous,
            wave_correction: self.cfd_proxy_wave,
            fn_ref: self.cfd_proxy_fn_ref,
        };

        // Planing override applies to the Holtrop path (as before).
        if self.resistance_model == ResistanceModel::HoltropMennen
            && speed >= self.planing_min_speed_kt
            && hull.displacement <= self.planing_max_displacement_tons
        {
            let planing_hp = crate::planing::planing_hp_estimate_calibrated(
                hull.length,
                hull.beam,
                hull.displacement * 2240.0,
                speed,
                self.planing_calibration_factor,
            );
            return cfd.apply(planing_hp, fn_) * self.power_factor;
        }

        let holtrop_hp = crate::resistance::holtrop_power_hp_calibrated(
            hull,
            speed,
            self.holtrop_viscous_factor,
            self.holtrop_wave_factor,
            self.holtrop_correlation_factor,
        ) * self.power_factor;

        let weight = self.blend_weight.clamp(0.0, 1.0);
        let base_hp = if weight < 1.0 - 1e-9 {
            weight * holtrop_hp + (1.0 - weight) * original_hp
        } else {
            match self.resistance_model {
                ResistanceModel::HoltropMennen => holtrop_hp,
                ResistanceModel::Original => original_hp,
            }
        };
        cfd.apply(base_hp, fn_)
    }

    pub fn resistance(&mut self, hull: &HullValues) {
        self.frict_max = 0.01 * hull.wetted_surface * pow(self.speed_max, 1.83);
        self.frict_cruise = 0.01 * hull.wetted_surface * pow(self.speed_cruise, 1.83);
        self.wave_max = pow(hull.displacement, 0.66666668653488159) / hull.length
            * hull.co_sharp
            * pow(self.speed_max, 4.0);
        self.wave_cruise = pow(hull.displacement, 0.66666668653488159) / hull.length
            * hull.co_sharp
            * pow(self.speed_cruise, 4.0);
        self.power_wave_max = if self.speed_max != 0.0 {
            self.wave_max / (self.wave_max + self.frict_max) * 100.0
        } else {
            0.0
        };
        self.power_wave_cruise = if self.speed_cruise != 0.0 {
            self.wave_cruise / (self.wave_cruise + self.frict_cruise) * 100.0
        } else {
            0.0
        };
        self.hp_max = self.speed_hp_calc(self.speed_max, hull);
        self.hp_cruise = self.speed_hp_calc(self.speed_cruise.min(self.speed_max), hull);
        // The propulsion-efficiency breakdown belongs to the Holtrop-Mennen
        // model.  The original empirical fit outputs shaft power directly and
        // does not use a QPC, so we must not display one for it.
        self.qpc_applicable = self.resistance_model == ResistanceModel::HoltropMennen;
        if self.qpc_applicable {
            let pe = crate::resistance::estimate_propulsion_efficiency(hull, self.speed_max);
            self.wake_fraction = pe.wake_fraction;
            self.thrust_deduction = pe.thrust_deduction;
            self.hull_efficiency = pe.hull_efficiency;
            self.open_water_efficiency = pe.open_water_efficiency;
            self.relative_rotative_efficiency = pe.relative_rotative_efficiency;
            self.shaft_efficiency = pe.shaft_efficiency;
            self.qpc = pe.quasi_propulsive_coefficient;
        } else {
            self.wake_fraction = 0.0;
            self.thrust_deduction = 0.0;
            self.hull_efficiency = 0.0;
            self.open_water_efficiency = 0.0;
            self.relative_rotative_efficiency = 0.0;
            self.shaft_efficiency = 0.0;
            self.qpc = 0.0;
        }
    }

    pub fn engine_calc(
        &mut self,
        hull: &HullValues,
        guns: &GunsAggregate,
        arm: &ArmourValues,
        weapons_misc: f64,
    ) {
        self.engine_recip_simple = 0.0;
        self.engine_recip_complex = 0.0;
        self.engine_turbine = 0.0;
        self.engine_count = 0;

        if self.engine_types[0] {
            self.engine_recip_simple = if hull.date_engine >= 1885 {
                if hull.date_engine >= 1950 {
                    4.075
                } else {
                    2.45 + (hull.date_engine - 1885) as f64 * 0.025
                }
            } else {
                1.2 + (hull.date_engine - 1860) as f64 * 0.05
            };
            self.engine_count += 1;
        }
        if self.engine_types[1] {
            self.engine_recip_complex = if hull.date_engine >= 1906 {
                if hull.date_engine >= 1911 {
                    if hull.date_engine >= 1950 {
                        8.5
                    } else {
                        7.5 + (hull.date_engine - 1910) as f64 * 0.025
                    }
                } else {
                    3.5 + (hull.date_engine - 1906) as f64
                }
            } else {
                1.2 + (hull.date_engine - 1860) as f64 * 0.05
            };
            self.engine_count += 1;
        }
        if self.engine_types[2] || self.fuel[3] || self.fuel[2] || self.fuel[4] {
            self.engine_turbine = if hull.date_engine >= 1898 {
                if hull.date_engine >= 1903 {
                    if hull.date_engine >= 1910 {
                        if hull.date_engine >= 1950 {
                            19.0
                        } else {
                            11.0 + (hull.date_engine - 1910) as f64 * 0.2
                        }
                    } else {
                        4.0 + (hull.date_engine - 1903) as f64
                    }
                } else {
                    1.0 + (hull.date_engine - 1898) as f64 * 0.5
                }
            } else {
                1.2 + (hull.date_engine - 1860) as f64 * 0.05
            };
            self.engine_count += 1;
        }

        let mut e = if self.engine_count > 0 {
            (self.engine_recip_simple + self.engine_recip_complex + self.engine_turbine)
                / self.engine_count as f64
                * (1.1000000238418579 - self.pct_coal / 1000.0)
        } else {
            0.0
        };
        if self.hp_max != 0.0 {
            e = self.hp_max / e;
        } else {
            e = 0.0;
        }
        if hull.date_engine < 1890 {
            e /= 1.0 + (1890.0 - hull.date_engine as f64) / 100.0;
        }
        if e.is_nan() || e.is_infinite() {
            e = 0.0;
        }
        self.engine = e;
        let denom = self.engine + guns.borne_weight * 8.0 + arm.weight + weapons_misc;
        self.disp_factor = if denom > 0.0 {
            (hull.displacement / denom).min(10.0)
        } else {
            10.0
        };
        self.engine_weight = self.engine / 2.0;
        if hull.displacement < 5000.0 && hull.displacement >= 600.0 && self.disp_factor < 1.0 {
            self.engine_weight *= pow(self.disp_factor, 1.0 - hull.displacement / 5000.0);
        } else if hull.displacement < 600.0 && self.disp_factor < 1.0 {
            self.engine_weight *= pow(self.disp_factor, 0.88);
        }
        let mw_factor =
            if self.machinery_weight_factor > 0.0 && self.machinery_weight_factor.is_finite() {
                self.machinery_weight_factor
            } else {
                1.0
            };
        self.engine_weight *= mw_factor;
    }

    pub fn bunker_calc(&mut self, hull: &HullValues) {
        let mut b = self.range / (1.0 + 0.40000000596046448 * (100.0 - self.pct_coal) / 100.0);
        if self.engine_types[0] || self.engine_types[1] {
            b /= 1.0 - (1910.0 - hull.date_engine as f64) / 70.0;
        } else if hull.date_engine < 1898 {
            b /= 1.0 - (1910.0 - hull.date_engine as f64) / 70.0;
        } else if hull.date_engine < 1920 {
            b /= 1.0 + (hull.date_engine as f64 - 1910.0) / 20.0;
        } else if hull.date_engine < 1950 {
            b /= 1.5 + (hull.date_engine as f64 - 1920.0) / 60.0;
        } else {
            b /= 2.0;
        }
        b /= 1.7999999523162842 / self.hp_cruise * RANGE * self.speed_cruise * 0.10000000149011612;
        b += hull.displacement * 0.005;
        if b.is_nan() || b.is_infinite() {
            b = 0.0;
        }
        self.bunker = b;
    }

    pub fn load_and_hull_weight(
        &mut self,
        hull: &HullValues,
        guns: &GunsAggregate,
        arm: &ArmourValues,
        weapons: &WeaponAggregate,
    ) {
        self.load_weight = hull.displacement * 0.02 + guns.magazine + self.bunker;
        self.hull_weight = hull.displacement
            - guns.guns_weight
            - weapons.weight
            - arm.weight
            - self.engine_weight
            - self.load_weight
            - weapons.misc_weight_vital
            - weapons.misc_weight_hull
            - weapons.misc_weight_deck
            - weapons.misc_weight_above
            - weapons.void_weight;
        self.hull_weight += guns.guns_weight - guns.borne_weight;
    }
}

/// Top-level ship design.
#[derive(Clone, Debug)]
pub struct ShipModel {
    pub name: String,
    pub country: String,
    pub ship_type: String,
    pub hull: HullValues,
    pub free: FreeboardValues,
    pub armour: ArmourValues,
    pub guns: GunsAggregate,
    pub weapons: WeaponAggregate,
    pub engine: EngineValues,
    pub steadiness: i32,
    pub space_crew_volume_per_person: f64,
    pub space_machinery_volume_per_kw: f64,
    pub space_stores_volume_per_ton: f64,
    pub planing_calibration_factor: f64,

    // performance results
    pub room_hull: f64,
    pub room_deck: f64,
    pub room_float: f64,
    pub stability: f64,
    pub stability_adj: f64,
    pub metacentre: f64,
    pub roll_period: f64,
    pub seaboat: f64,
    pub steadiness_adj: f64,
    pub seakeeping: f64,
    pub recoil: f64,
    pub struct_weight: f64,
    pub disp_max_load: f64,
    pub disp_standard: f64,
    pub disp_light: f64,
    pub block_deep: f64,
    pub cost_dollars: f64,
    pub cost_pounds: f64,
    pub concentration: f64,
    pub strength_cross: f64,
    pub strength_long: f64,
    pub strength_comp: f64,
    pub flotation: f64,
    pub damage_shell: f64,
    pub damage_torp: f64,
    pub super_factor_long: f64,
    pub date_adj_float: f64,
    /// Whether the armour <-> engine-weight fixed-point iteration converged.
    pub converged: bool,
    /// Number of fixed-point iterations actually performed.
    pub iterations: usize,
    /// Final relative residual of the fixed-point cycle (armour weight change).
    pub iteration_residual: f64,
}

impl Default for ShipModel {
    fn default() -> Self {
        Self {
            name: "Enter ship name".into(),
            country: "Enter country".into(),
            ship_type: "Enter ship type".into(),
            hull: HullValues::default(),
            free: FreeboardValues {
                fc_length: 0.2,
                fd_length: 0.3,
                qd_length: 0.15,
                ..Default::default()
            },
            armour: ArmourValues::default(),
            guns: GunsAggregate::default(),
            weapons: WeaponAggregate::default(),
            engine: EngineValues {
                power_factor: 1.0,
                original_power_factor: 1.0,
                blend_weight: 1.0,
                holtrop_viscous_factor: 1.0,
                holtrop_wave_factor: 1.0,
                holtrop_correlation_factor: 1.0,
                planing_calibration_factor: 1.0,
                cfd_proxy_enabled: false,
                cfd_proxy_viscous: 0.0,
                cfd_proxy_wave: 0.0,
                cfd_proxy_fn_ref: 0.25,
                planing_min_speed_kt: 25.0,
                planing_max_displacement_tons: 3000.0,
                ensemble_reduced_weight: 0.5,
                ensemble_structural_weight: 0.5,
                machinery_weight_factor: 1.0,
                ..EngineValues::default()
            },
            steadiness: 50,
            space_crew_volume_per_person: 0.0,
            space_machinery_volume_per_kw: 0.0,
            space_stores_volume_per_ton: 0.0,
            planing_calibration_factor: 1.0,
            room_hull: 0.0,
            room_deck: 0.0,
            room_float: 0.0,
            stability: 0.0,
            stability_adj: 0.0,
            metacentre: 0.0,
            roll_period: 0.0,
            seaboat: 0.0,
            steadiness_adj: 0.0,
            seakeeping: 0.0,
            recoil: 0.0,
            struct_weight: 0.0,
            disp_max_load: 0.0,
            disp_standard: 0.0,
            disp_light: 0.0,
            block_deep: 0.0,
            cost_dollars: 0.0,
            cost_pounds: 0.0,
            concentration: 1.0,
            strength_cross: 0.0,
            strength_long: 0.0,
            strength_comp: 0.0,
            flotation: 0.0,
            damage_shell: 0.0,
            damage_torp: 0.0,
            super_factor_long: 1.0,
            date_adj_float: 1.0,
            converged: false,
            iterations: 0,
            iteration_residual: f64::NAN,
        }
    }
}

/// Compute `(strength_cross, strength_long, strength_comp)`.
fn strengths(
    hull: &HullValues,
    free: &FreeboardValues,
    arm: &ArmourValues,
    guns: &GunsAggregate,
    engine: &EngineValues,
    struct_weight: f64,
    super_factor_long: f64,
    concentration: f64,
) -> (f64, f64, f64) {
    let date_factor = if hull.date_hull < 1900 {
        1.0 - (1900.0 - hull.date_hull as f64) / 100.0
    } else {
        1.0
    };

    let mut cross = struct_weight
        / (hull.beam_bulges * (hull.draught + free.free_distributed))
            .max(0.0)
            .sqrt()
        / (((hull.displacement
            + ((guns.broadside + guns.borne_weight + guns.arm_weight + arm.weight_conning)
                * (concentration * guns.super_factor)
                + engine.hp_max.max(0.0) / 100.0))
            / hull.displacement.max(f64::EPSILON))
            * 0.6);
    cross *= date_factor;

    let long_base = if arm.bulkhead_type != 1 || arm.weight_bulkhead <= 0.0 {
        engine.hull_weight
    } else {
        engine.hull_weight + arm.weight_bulkhead
    };
    let mut long = long_base
        / (pow(
            hull.length / (hull.draught + free.freeboard_cap).max(f64::EPSILON),
            2.0,
        ) * (hull.displacement
            + arm.weight_belt_ends * 3.0
            + (guns.borne_weight + guns.arm_weight) * super_factor_long * 2.0))
        * 850.0;
    long *= date_factor;

    let comp = if cross <= long {
        cross * pow(long / cross.max(f64::EPSILON), 0.1)
    } else {
        long * pow(cross / long.max(f64::EPSILON), 0.25)
    }
    .min(1.0e6);
    (cross, long, comp)
}

impl ShipModel {
    pub fn new() -> Self {
        Self::default()
    }

    fn prepare_hull_freeboard(&mut self) {
        self.date_adj_float = if self.hull.date_hull < 1850 || self.hull.date_hull > 1950 {
            0.0
        } else if self.hull.date_hull <= 1890 {
            1.0 - (1890 - self.hull.date_hull) as f64 / 66.6666641
        } else {
            1.0
        };

        // Length over all (HullControl.LengthCalc)
        self.free.length_stem =
            self.free.free_fc_for * (self.free.length_bow * std::f64::consts::PI / 180.0).tan();
        self.hull.length_oa = self.hull.length;
        if self.free.length_ram > 0.0 && self.free.bow > 1 {
            self.hull.length_oa += self.free.length_stem.max(self.free.length_ram);
        } else if self.free.length_bow > 0.0 {
            self.hull.length_oa += self.free.length_stem;
        }
        if self.free.length_stern > 0.0 {
            self.hull.length_oa += self.free.length_stern;
        }

        self.free.calculate();
        self.hull.transom_width = self.free.transom_width;
        self.hull.screws = self.engine.fuel.iter().any(|x| *x) as i32; // placeholder, set from engine screws below
        self.hull.stern = self.free.stern;
        self.hull.calculate(&self.free);
    }

    /// Recalculate the complete design. The C# program performs the same
    /// calculations on every UI event; this method runs the network of
    /// dependencies to convergence.
    pub fn calculate(&mut self) {
        self.engine.planing_calibration_factor = self.planing_calibration_factor;
        // First pass: hull/freeboard/weapons/guns.
        self.prepare_hull_freeboard();
        self.hull.screws = self.engine.screws;
        self.hull.calculate(&self.free);
        self.ensure_hull_defaults();
        self.weapons.calculate(self.hull.date_hull, &self.hull);
        self.guns.calculate(&self.free);
        self.free
            .calculate_cap(self.hull.beam, &self.guns.batteries);

        // Fixed-point iteration for the armour <-> engine weight cycle.
        //
        // The cycle variable is the total armour weight: each pass recomputes
        // armour from the current machinery weight, then machinery from the
        // new armour.  Iterate until the relative change falls below
        // `FIXED_POINT_TOLERANCE`, with a hard cap so a genuinely divergent
        // design still terminates (and is reported as not converged).
        const MAX_FIXED_POINT_ITERATIONS: usize = 50;
        const FIXED_POINT_TOLERANCE: f64 = 1e-9;

        let mut previous_armour_weight = f64::NAN;
        let mut iterations = 0_usize;
        let mut residual = f64::INFINITY;
        let mut converged = false;
        for _ in 0..MAX_FIXED_POINT_ITERATIONS {
            iterations += 1;
            self.armour.calculate(
                &self.hull,
                &self.free,
                &self.guns,
                self.engine.engine_weight,
            );
            self.engine.resistance(&self.hull);
            self.engine.engine_calc(
                &self.hull,
                &self.guns,
                &self.armour,
                self.weapons.misc_weight_vital
                    + self.weapons.misc_weight_hull
                    + self.weapons.misc_weight_deck
                    + self.weapons.misc_weight_above
                    + self.weapons.void_weight,
            );
            self.engine.bunker_calc(&self.hull);
            self.engine
                .load_and_hull_weight(&self.hull, &self.guns, &self.armour, &self.weapons);

            let armour_weight = self.armour.weight;
            if previous_armour_weight.is_finite() {
                let scale = armour_weight
                    .abs()
                    .max(previous_armour_weight.abs())
                    .max(1e-12);
                residual = (armour_weight - previous_armour_weight).abs() / scale;
                if residual < FIXED_POINT_TOLERANCE {
                    converged = true;
                    break;
                }
            }
            previous_armour_weight = armour_weight;
        }
        self.converged = converged;
        self.iterations = iterations;
        self.iteration_residual = residual;

        // Final resistance / machinery values.
        self.engine.resistance(&self.hull);
        self.engine.engine_calc(
            &self.hull,
            &self.guns,
            &self.armour,
            self.weapons.misc_weight_vital
                + self.weapons.misc_weight_hull
                + self.weapons.misc_weight_deck
                + self.weapons.misc_weight_above
                + self.weapons.void_weight,
        );
        self.engine.bunker_calc(&self.hull);
        self.engine
            .load_and_hull_weight(&self.hull, &self.guns, &self.armour, &self.weapons);

        // Performance block.
        self.calculate_performance();
    }

    fn ensure_hull_defaults(&mut self) {
        if self.hull.lcb_percent == 0.0 {
            self.hull.lcb_percent = -0.75;
        }
        if self.hull.midship_coefficient == 0.0 {
            self.hull.midship_coefficient = (0.90 + 0.10 * self.hull.block).min(0.99);
        }
        if self.hull.bulb_area_sqft == 0.0 && self.hull.date_hull >= 1910 {
            // Default bulbous bow for modern large ships: roughly 2% of the
            // immersed midship area.  Users can override this.
            let midship_area = self.hull.beam * self.hull.draught;
            self.hull.bulb_area_sqft = (0.02 * midship_area).max(0.0);
        }
        if self.hull.transom_area_sqft == 0.0 && self.free.stern == 1 {
            self.hull.transom_area_sqft =
                (self.free.transom_width * self.hull.beam * self.hull.draught).max(0.0);
        }
        if self.hull.appendage_area_sqft == 0.0 {
            self.hull.appendage_area_sqft = 0.04 * self.hull.wetted_surface.max(0.0);
        }
        if self.hull.stern_shape == 0.0 {
            self.hull.stern_shape = if self.free.stern == 1 { -10.0 } else { 0.0 };
        }
    }

    fn calculate_performance(&mut self) {
        let h = &self.hull;
        let f = &self.free;
        let a = &self.armour;
        let g = &self.guns;
        let w = &self.weapons;
        let e = &self.engine;

        self.room_float = (g.magazine
            + h.displacement * 0.019999999552965164
            + g.borne_weight * 6.4000000953674316
            + e.engine_weight * 3.0
            + w.misc_weight_vital
            + w.misc_weight_hull)
            / (h.displacement * 0.93999999761581421)
            / (1.0 - w.space_hull);
        self.room_hull = if a.weight_bulkhead <= 0.1 {
            self.room_float
        } else {
            self.room_float * h.beam / a.bulkhead_beam
        };
        self.room_deck = h.waterplane_area / TON / 15.0 * (1.0 - w.space_deck)
            / h.crew_min.max(1) as f64
            * f.free_distributed;

        // Stability.
        let mut stability = a.weight_conning * 5.0
            + (g.borne_weight + g.arm_weight) * (2.0 * g.super_factor - 1.0) * 4.0
            + w.misc_weight_hull * 2.0
            + w.misc_weight_deck * 3.0
            + w.misc_weight_above * 4.0
            + a.weight_belt_upper * 2.0
            + a.weight_belt_main
            + a.weight_belt_ends
            + a.weight_deck
            + (e.hull_weight + g.guns_weight - g.borne_weight) * 1.5 * f.freeboard / h.draught;
        if self.room_deck < 1.0 {
            stability += (e.engine_weight + w.misc_weight_vital + w.void_weight)
                * pow(1.0 - self.room_deck, 2.0);
        }
        self.stability = if stability > 0.0 {
            pow(
                h.displacement * (h.beam_bulges / h.draught) / stability * 0.5,
                0.5,
            ) * pow(8.76755 / h.length_beam, 0.25)
        } else {
            0.0
        };
        self.stability_adj = self.stability * ((50.0 - self.steadiness as f64) / 150.0 + 1.0);
        self.metacentre = pow(h.beam, 1.5) * (self.stability_adj - 0.5) / 0.5 / 200.0;
        self.roll_period = if self.metacentre > 0.0 {
            0.42 * h.beam_bulges / self.metacentre.sqrt()
        } else {
            0.0
        };

        // Super-factor longitudinal (only uses first battery in the original).
        self.super_factor_long = self.room_hull;
        let b1 = &g.batteries[0];
        let b1_layouts =
            b1.layout_g1 == 0 || b1.layout_g1 == 9 || b1.layout_g2 == 0 || b1.layout_g2 == 9;
        if b1_layouts && (b1.mounts == 3 || b1.mounts == 4) {
            self.super_factor_long *= g.super_factor;
        } else if (b1.mounts_g1 > 0
            && b1.mounts_g2 == 0
            && ((3..=8).contains(&b1.layout_g1) || (12..=17).contains(&b1.layout_g1)))
            || (b1.mounts_g2 > 0
                && b1.mounts_g1 == 0
                && ((3..=8).contains(&b1.layout_g2) || (12..=17).contains(&b1.layout_g2)))
            || (b1.mounts_g1 > 0
                && b1.mounts_g2 > 0
                && (b1.position_g1 - b1.position_g2).abs() < 0.2)
        {
            self.super_factor_long *= g.super_factor * 0.8;
        } else {
            self.super_factor_long *= 2.0 * g.super_factor - 1.0;
        }

        // Seaboat/steadiness/seakeeping.
        let mut seaboat = (f.freeboard_cap / (2.4 * pow(h.displacement, 0.2)))
            .max(0.0)
            .sqrt()
            * (pow(self.stability * 5.0 * (h.beam_bulges / h.length), 0.2)
                * (f.freeboard_cap / h.length * 20.0).max(0.0).sqrt()
                * (h.displacement
                    / (h.displacement
                        + a.weight_belt_ends * 3.0
                        + e.hull_weight / 3.0
                        + (g.borne_weight + g.arm_weight) * self.super_factor_long)))
            * 8.0;
        if h.draught / h.beam_bulges < 0.3 {
            seaboat *= (h.draught / h.beam_bulges / 0.3).max(0.0).sqrt();
        }
        let frict_frac = e.frict_max / (e.frict_max + e.wave_max);
        if frict_frac < 0.55 && e.speed_max != 0.0 {
            seaboat *= pow(frict_frac, 2.0);
        } else {
            seaboat *= 0.3025;
        }
        self.seaboat = seaboat.min(2.0);
        let steadiness_adj = (self.steadiness as f64 * self.seaboat).min(100.0);
        self.steadiness_adj = steadiness_adj;
        self.seakeeping = if steadiness_adj >= 50.0 {
            self.seaboat
        } else {
            self.seaboat * steadiness_adj / 50.0
        };

        // Recoil.
        self.recoil =
            g.broadside / h.displacement.max(f64::EPSILON) * f.free_distributed * g.super_factor
                / h.beam_bulges
                * pow(
                    pow(h.displacement, 0.3333333432674408) / h.beam_bulges * 3.0,
                    2.0,
                )
                * 7.0;
        if self.stability_adj > 0.0 {
            self.recoil /= self.stability_adj * ((50.0 - steadiness_adj) / 150.0 + 1.0);
        }

        // Structural weight.
        self.struct_weight = if a.bulkhead_type != 1 || a.weight_bulkhead <= 0.0 {
            e.hull_weight
        } else {
            e.hull_weight + a.weight_bulkhead
        };
        self.struct_weight *=
            POUND2TON / (h.wetted_surface + 2.0 * h.length * f.freeboard_cap + h.waterplane_area);

        // Displacements.
        self.disp_max_load = h.displacement + 0.8 * e.bunker;
        let draught_deep = h.draught
            + (self.disp_max_load - h.displacement) / (h.waterplane_area / TON).max(f64::EPSILON);
        self.block_deep =
            (self.disp_max_load / (h.length * h.beam_bulges * draught_deep / TON)).min(1.0);
        self.disp_standard = h.displacement - e.bunker;
        self.disp_light = h.displacement - e.load_weight;

        // Cost.
        self.cost_dollars = (h.displacement - e.load_weight) * 0.00014
            + e.engine_weight * 0.00056
            + g.borne_weight * 8.0 * 0.00042;
        if h.date_hull + 2 > 1914 {
            self.cost_dollars *= 1.0 + (h.date_hull as f64 + 1.5 - 1914.0) / 5.5;
        }
        self.cost_pounds = self.cost_dollars / 4.0;

        // Strength.
        self.concentration = 1.0;
        if g.broadside > 0.0 {
            for b in g.batteries.iter() {
                let mut mounts = b.mounts;
                if mounts == 0 {
                    mounts = b.barrels;
                }
                let mut conc = if mounts != 0 {
                    b.shell_weight * b.barrels as f64 / g.broadside
                } else {
                    0.0
                };
                if b.barrels > 0 {
                    if b.mount_adj > 0.6 {
                        conc *= pow(4.0 / mounts as f64, 0.25) - 1.0;
                    } else {
                        conc *= -0.1;
                    }
                } else {
                    conc = 0.0;
                }
                self.concentration += conc;
            }
        }
        let (cross, long, comp) = strengths(
            h,
            f,
            a,
            g,
            e,
            self.struct_weight,
            self.super_factor_long,
            self.concentration,
        );
        self.strength_cross = cross;
        self.strength_long = long;
        self.strength_comp = comp;

        // Flotation and damage.
        let has_submerged = g
            .batteries
            .iter()
            .any(|b| b.mount_type == 0 && b.gun_sub_g1 + b.gun_sub_g2 > 0);
        let mut flotation = if self.strength_comp >= 0.0 {
            let base = if has_submerged {
                f.freeboard_cap
            } else {
                f.free_distributed
            };
            let mut fl = (base * h.waterplane_area / TON + h.displacement) / 2.0;
            if self.stability_adj > 1.0 {
                fl *= self.stability_adj.sqrt();
            } else {
                fl *= pow(self.stability_adj, 4.0);
            }
            if self.strength_comp < 1.0 {
                fl *= self.strength_comp;
            }
            fl
        } else {
            0.0
        };
        if self.room_float > 1.0 {
            flotation /= pow(self.room_float, 2.0);
        } else {
            flotation /= self.room_float.max(f64::EPSILON);
        }
        flotation *= self.date_adj_float;
        self.flotation = flotation;

        let primary_calibre = if g.batteries[0].calibre > 0.0 {
            g.batteries[0].calibre
        } else {
            6.0
        };
        self.damage_shell =
            self.flotation / (pow(primary_calibre, 3.0) / 2.0 * self.date_adj_float);

        let mut dt = (pow(flotation / 10000.0, 0.3333333432674408)
            + pow(h.beam_bulges / 75.0, 2.0)
            + pow(
                a.bulkhead / 2.0 * a.bulkhead_length / h.length / 0.65 * a.bulkhead_height
                    / h.draught.max(f64::EPSILON),
                0.3333333432674408,
            ) * flotation
                / 35000.0
                * h.beam_bulges
                / 50.0)
            / self.room_float.max(f64::EPSILON)
            * h.length
            / (h.length + h.beam_bulges);
        if self.stability_adj < 1.0 {
            dt *= pow(self.stability_adj, 4.0);
        }
        dt *= 1.0 - w.space_hull;
        if w.torpedo1.weapon_weight > 0.0 && w.torpedo1.number > 0 {
            dt *= 1.312999963760376 / (w.torpedo1.weapon_weight / w.torpedo1.number as f64);
        }
        self.damage_torp = dt;
        self.hull.draught_deep = draught_deep;
    }
}

/// Human-readable summary of the most important design results.
pub struct Summary {
    pub lines: Vec<String>,
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in &self.lines {
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

impl ShipModel {
    pub fn summary(&self) -> Summary {
        let g = &self.guns;
        let e = &self.engine;
        let h = &self.hull;
        let mut lines = vec![
            format!("Ship: {} ({}, {})", self.name, self.country, self.ship_type),
            format!("Normal displacement: {:.0} tons", h.displacement),
            format!(
                "Dimensions: {:.1} x {:.1} x {:.1} ft, block coefficient {:.3}",
                h.length, h.beam_bulges, h.draught, h.block
            ),
            format!(
                "Freeboard: {:.2} ft (distributed {:.2} ft)",
                self.free.freeboard, self.free.free_distributed
            ),
            format!(
                "Speed: max {:.1} kt, cruise {:.1} kt",
                e.speed_max, e.speed_cruise
            ),
            format!(
                "Power: max {:.0} hp, cruise {:.0} hp",
                e.hp_max, e.hp_cruise
            ),
            if e.qpc_applicable {
                format!(
                    "Propulsion: QPC={:.3} (wake={:.3}, thrust deduction={:.3}); used in power calculation",
                    e.qpc, e.wake_fraction, e.thrust_deduction
                )
            } else {
                "Propulsion: original empirical shaft-power fit; no explicit QPC (the Holtrop-style efficiency breakdown is not used)."
                    .to_string()
            },
            match self.engine.resistance_model {
                ResistanceModel::Original => {
                    "Note: power uses the original SpringSharp empirical model (reconstructed, unimproved)."
                        .to_string()
                }
                ResistanceModel::HoltropMennen => {
                    "Note: power uses the improved Holtrop-Mennen-style model.".to_string()
                }
            },
            format!("Engine weight: {:.0} tons", e.engine_weight),
            format!(
                "Bunker: {:.0} tons normal, {:.0} tons max",
                e.bunker,
                e.bunker * 1.8
            ),
            format!("Load weight: {:.0} tons", e.load_weight),
            format!("Hull weight: {:.0} tons", e.hull_weight),
            format!(
                "Guns: {:.0} tons gun/mount + {:.0} tons gun armour",
                g.guns_weight, g.arm_weight
            ),
            format!(
                "Magazine: {:.0} tons, broadside {:.0} lbs",
                g.magazine, g.broadside
            ),
            format!(
                "Armour: belts {:.0}, decks {:.0}, conning {:.0}, total {:.0} tons",
                self.armour.weight_belts,
                self.armour.weight_deck,
                self.armour.weight_conning,
                self.armour.weight
            ),
            format!("Secondary weapons: {:.0} tons", self.weapons.weight),
            format!(
                "Room: hull {:.1}%, deck {:.1}%",
                self.room_hull * 100.0,
                self.room_deck * 100.0
            ),
            format!(
                "Stability: {:.2}, metacentre {:.2} ft",
                self.stability_adj, self.metacentre
            ),
            format!(
                "Seakeeping: {:.2}, steadiness {:.0}",
                self.seakeeping, self.steadiness_adj
            ),
            format!("Recoil: {:.2}", self.recoil),
            format!("Structural weight: {:.0} lbs/sq ft", self.struct_weight),
            format!(
                "Strength cross/long/composite: {:.2}/{:.2}/{:.2}",
                self.strength_cross, self.strength_long, self.strength_comp
            ),
            format!(
                "Cost: £{:.3}M / ${:.3}M",
                self.cost_pounds, self.cost_dollars
            ),
            format!(
                "Flotation: {:.0} tons; shell hits {:.1}; torpedo hits {:.1}",
                self.flotation, self.damage_shell, self.damage_torp
            ),
        ];
        if self.strength_comp < 0.5 {
            lines.push(
                "DESIGN FAILURE: Hull structure insufficient to carry required loads!".into(),
            );
        } else if self.strength_comp < 0.995 {
            lines.push("WARNING: Hull structure strained by open sea conditions!".into());
        }
        // The armour <-> machinery cycle is a fixed-point iteration.  If it hits
        // its iteration cap the reported weights are still the last iterate, but
        // they are no longer trustworthy to the usual tolerance, so say so
        // instead of silently reporting them as settled.  Guarded on
        // `iterations > 0` so a model that has never been calculated (where the
        // flag is still at its default) does not warn spuriously.
        if self.iterations > 0 && !self.converged {
            lines.push(format!(
                "WARNING: armour/machinery weight iteration did not converge after {} iterations (relative residual {:.2e}); the weight breakdown is approximate.",
                self.iterations, self.iteration_residual
            ));
        }
        Summary { lines }
    }
}

// Convenience accessors used by the CLI.
impl ShipModel {
    pub fn battery_mut(&mut self, index: usize) -> &mut GunBattery {
        &mut self.guns.batteries[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hull_displacement_matches_springsharp_formula() {
        let mut hull = HullValues {
            length: 600.0,
            beam: 90.0,
            beam_bulges: 90.0,
            draught: 28.0,
            block: 0.55,
            date_hull: 1915,
            date_engine: 1915,
            screws: 4,
            transom_width: 0.0,
            stern: 0,
            ..Default::default()
        };
        hull.calculate(&FreeboardValues::default());
        assert!((hull.displacement - 23760.0).abs() < 1.0);
        assert!(hull.waterplane_area > 0.0);
        assert!(hull.wetted_surface > 0.0);
    }

    #[test]
    fn freeboard_distribution_is_weighted() {
        let mut free = FreeboardValues {
            fc_length: 0.2,
            fd_length: 0.3,
            qd_length: 0.15,
            free_fc_for: 22.0,
            free_fc_aft: 18.0,
            free_fd_for: 18.0,
            free_fd_aft: 15.0,
            free_ad_for: 15.0,
            free_ad_aft: 12.0,
            free_qd_for: 12.0,
            free_qd_aft: 10.0,
            ..Default::default()
        };
        free.calculate();
        assert!((free.ad_length - 0.35).abs() < 1e-9);
        assert!(free.freeboard > 0.0);
        assert!(free.free_distributed > 0.0);
    }

    #[test]
    fn shell_weight_known_calibre() {
        let mut battery = GunBattery {
            calibre: 15.0,
            cal_length: 45.0,
            date_gun_adj: 1.0,
            ..Default::default()
        };
        let weight = battery.shell_weight_calc();
        assert!((weight - 15.0_f64.powi(3) / 1.9830943211886).abs() < 0.01);
    }

    #[test]
    fn qpc_is_marked_applicable_only_for_holtrop() {
        let mut ship = ShipModel::new();
        ship.hull.length = 600.0;
        ship.hull.beam = 90.0;
        ship.hull.beam_bulges = 90.0;
        ship.hull.draught = 28.0;
        ship.hull.block = 0.55;
        ship.hull.date_hull = 1940;
        ship.hull.date_engine = 1940;
        ship.engine.screws = 4;
        ship.engine.speed_max = 24.0;
        ship.engine.resistance_model = ResistanceModel::Original;
        ship.calculate();
        assert!(!ship.engine.qpc_applicable);
        assert_eq!(ship.engine.qpc, 0.0);

        ship.engine.resistance_model = ResistanceModel::HoltropMennen;
        ship.calculate();
        assert!(ship.engine.qpc_applicable);
        assert!(ship.engine.qpc > 0.0);
        assert!(ship.engine.hull_efficiency > 0.0);
    }

    #[test]
    fn blend_weight_mixes_the_two_models() {
        let mut ship = ShipModel::new();
        ship.hull.length = 600.0;
        ship.hull.beam = 90.0;
        ship.hull.beam_bulges = 90.0;
        ship.hull.draught = 28.0;
        ship.hull.block = 0.55;
        ship.hull.date_hull = 1940;
        ship.hull.date_engine = 1940;
        ship.engine.screws = 4;
        ship.engine.speed_max = 24.0;
        ship.engine.power_factor = 1.0;
        ship.engine.original_power_factor = 1.0;

        ship.engine.resistance_model = ResistanceModel::Original;
        ship.engine.blend_weight = 1.0;
        ship.calculate();
        let original = ship.engine.hp_max;

        ship.engine.resistance_model = ResistanceModel::HoltropMennen;
        ship.engine.blend_weight = 1.0;
        ship.calculate();
        let holtrop = ship.engine.hp_max;

        ship.engine.blend_weight = 0.5;
        ship.calculate();
        let blended = ship.engine.hp_max;
        assert!(blended > original.min(holtrop) && blended < original.max(holtrop));
        assert!((blended - 0.5 * (original + holtrop)).abs() < 1e-6);
    }

    #[test]
    fn planing_thresholds_are_preserved() {
        let mut ship = ShipModel::new();
        ship.engine.planing_min_speed_kt = 30.0;
        ship.engine.planing_max_displacement_tons = 5000.0;
        ship.hull.length = 100.0;
        ship.hull.beam = 20.0;
        ship.hull.beam_bulges = 20.0;
        ship.hull.draught = 5.0;
        ship.hull.block = 0.4;
        ship.hull.date_hull = 1940;
        ship.hull.date_engine = 1940;
        ship.engine.screws = 1;
        ship.calculate();
        assert_eq!(ship.engine.planing_min_speed_kt, 30.0);
        assert_eq!(ship.engine.planing_max_displacement_tons, 5000.0);
    }

    #[test]
    fn small_fast_hull_uses_planing_auto_switch() {
        let mut ship = ShipModel::new();
        ship.engine.resistance_model = ResistanceModel::HoltropMennen;
        ship.engine.planing_calibration_factor = 0.668;
        ship.hull.length = 80.0;
        ship.hull.beam = 20.0;
        ship.hull.beam_bulges = 20.0;
        ship.hull.draught = 4.0;
        ship.hull.block = 0.42;
        ship.hull.date_hull = 1940;
        ship.hull.date_engine = 1940;
        ship.free.free_fd_for = 5.0;
        ship.free.free_fd_aft = 4.0;
        ship.free.free_ad_for = 4.0;
        ship.free.free_ad_aft = 3.0;
        ship.engine.screws = 1;
        ship.engine.speed_max = 40.0;
        ship.engine.speed_cruise = 20.0;
        ship.engine.range = 1000.0;
        ship.calculate();
        assert!(ship.engine.hp_max.is_finite());
        assert!(ship.engine.hp_max > 0.0);
    }

    #[test]
    fn ship_calculation_completes() {
        let mut ship = ShipModel::new();
        ship.hull.length = 400.0;
        ship.hull.beam = 60.0;
        ship.hull.beam_bulges = 60.0;
        ship.hull.draught = 20.0;
        ship.hull.block = 0.5;
        ship.hull.date_hull = 1920;
        ship.hull.date_engine = 1920;
        ship.free.free_fd_for = 20.0;
        ship.free.free_fd_aft = 16.0;
        ship.free.free_ad_for = 16.0;
        ship.free.free_ad_aft = 14.0;
        ship.engine.fuel[0] = true;
        ship.engine.engine_types[1] = true;
        ship.engine.drive[0] = true;
        ship.engine.speed_max = 20.0;
        ship.engine.speed_cruise = 10.0;
        ship.engine.range = 7000.0;
        ship.engine.screws = 2;
        ship.calculate();
        assert!(ship.hull.displacement > 0.0);
        assert!(ship.strength_comp >= 0.0);
    }

    #[test]
    fn fixed_point_converges_and_reports_status() {
        let mut ship = crate::features::demo_ship();
        ship.calculate();
        assert!(ship.converged, "demo design should converge");
        assert!(ship.iterations >= 1);
        assert!(ship.iterations <= 50);
        assert!(ship.iteration_residual.is_finite());
        assert!(ship.iteration_residual < 1e-9);
    }

    #[test]
    fn fixed_point_convergence_is_stable() {
        // Re-running calculate() must produce the same converged state and the
        // same armour weight: the fixed point is deterministic.
        let mut a = crate::features::demo_ship();
        a.calculate();
        let weight_a = a.armour.weight;
        let iters_a = a.iterations;
        a.calculate();
        assert_eq!(a.iterations, iters_a);
        assert!((a.armour.weight - weight_a).abs() <= 1e-9 * weight_a.abs().max(1.0));
    }

    #[test]
    fn convergence_status_is_surfaced_in_summary_and_report() {
        let mut ship = crate::features::demo_ship();
        ship.calculate();
        assert!(!ship.summary().to_string().contains("did not converge"));
        assert!(!crate::features::generate_report(&ship).contains("did not converge"));

        // Force the not-converged state directly: building a design that
        // genuinely diverges is not portable, and the reporting path is what is
        // under test here.
        ship.converged = false;
        ship.iterations = 50;
        ship.iteration_residual = 1e-3;
        assert!(ship.summary().to_string().contains("did not converge"));
        let report = crate::features::generate_report(&ship);
        assert!(report.contains("did not converge"));
        assert!(report.contains("50 iterations"));

        // A model that has never been calculated must not warn spuriously.
        let fresh = ShipModel::new();
        assert!(!fresh.summary().to_string().contains("did not converge"));
        assert!(!crate::features::generate_report(&fresh).contains("did not converge"));
    }
}
