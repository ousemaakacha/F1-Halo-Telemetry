// Native controls and GDI decoration share this geometry so they reflow together.
use super::theme::Page;

pub type Bounds = (i32, i32, i32, i32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResponsiveLayout {
    pub width: i32,
    pub height: i32,
    pub content_height: i32,
    pub main_x: i32,
    pub main_width: i32,
    pub header_height: i32,
    pub page_top: i32,
    pub general_top: i32,
    pub general_height: i32,
    pub footer_top: i32,
    pub wide: bool,
    pub compact_nav: bool,
    pub single_column: bool,
    page: Page,
    scale_y: f32,
    body_shift: i32,
    nav_columns: i32,
}

impl ResponsiveLayout {
    pub fn new(width: i32, height: i32, page: Page) -> Self {
        let width = width.max(320);
        let height = height.max(1);
        let wide = width >= 1120;
        let compact_nav = width < 850;
        let main_x = if compact_nav { 24 } else { 244 };
        let main_width = width - main_x - if wide { 316 } else { 24 };
        let single_column = main_width < 580;
        let header_height = if compact_nav && width < 600 { 144 } else { 86 };
        let nav_columns = if width >= 520 { 3 } else { 2 };
        let page_top = if compact_nav {
            header_height + 20 + (6 / nav_columns) * 54 - 8 + 28
        } else {
            114
        };
        let body_shift = page_top - 114 + if single_column { 12 } else { 0 };
        let extra = if single_column {
            match page {
                Page::Layout => 348,
                Page::Tyres => 328,
                Page::Colours => 438,
                Page::Connection => 86,
                Page::Diagnostics => 32,
                Page::Shortcuts => 0,
            }
        } else {
            0
        };
        let general_top = 688 + body_shift + extra;
        let general_height = if main_width >= 560 { 400 } else { 628 };
        let content_height = if wide {
            height.max(660)
        } else {
            height.max(general_top + general_height + 258)
        };
        let scale_y = if wide {
            ((content_height - 86) as f32 / 694.0).min(1.0)
        } else {
            1.0
        };
        let footer_top = if wide {
            content_height - (80.0 * scale_y).round() as i32
        } else {
            content_height - 194
        };
        Self {
            width,
            height,
            content_height,
            main_x,
            main_width,
            header_height,
            page_top,
            general_top,
            general_height,
            footer_top,
            wide,
            compact_nav,
            single_column,
            page,
            scale_y,
            body_shift,
            nav_columns,
        }
    }

    fn y(&self, y: i32) -> i32 {
        if y < 86 {
            y
        } else {
            86 + ((y - 86) as f32 * self.scale_y).round() as i32
        }
    }

    fn main(&self, (x, y, w, h): Bounds) -> Bounds {
        let map_x = |x| {
            if !self.single_column {
                self.main_x + (x - 244) * self.main_width / 620
            } else if x <= 264 {
                self.main_x + x - 244
            } else if x >= 844 {
                self.main_x + self.main_width + x - 864
            } else {
                self.main_x + 20 + (x - 264) * (self.main_width - 40) / 580
            }
        };
        (
            map_x(x),
            if self.wide {
                self.y(y)
            } else {
                y + self.body_shift
            },
            map_x(x + w) - map_x(x),
            if self.wide {
                (h as f32 * self.scale_y).round() as i32
            } else {
                h
            },
        )
    }

    fn card_column(&self, (x, y, w, h): Bounds, left: i32, dy: i32) -> Bounds {
        (
            self.main_x + (x - left) * self.main_width / 300,
            y + self.body_shift + dy,
            w * self.main_width / 300,
            h,
        )
    }

    pub fn map(&self, bounds: Bounds) -> Bounds {
        let (x, y, w, h) = bounds;
        if bounds == (0, 0, 216, 780) {
            return if self.compact_nav {
                (0, 0, self.width, self.header_height)
            } else {
                (0, 0, 216, self.content_height)
            };
        }
        if x == 0 && y == 85 {
            return (0, self.header_height - 1, self.width, h);
        }
        if x == 216 && w == 0 {
            return if self.compact_nav {
                (0, 0, 0, 0)
            } else {
                (216, 0, 0, self.content_height)
            };
        }
        if x == 892 && y == 86 {
            return if self.wide {
                (self.width - 288, 86, w, self.content_height - 86)
            } else if w == 0 {
                (0, 0, 0, 0)
            } else {
                (
                    self.main_x,
                    self.general_top,
                    self.main_width,
                    self.general_height,
                )
            };
        }
        if y == 724 {
            if self.wide {
                let (nx, nw) = if x < 216 {
                    (x, w)
                } else if x >= 892 {
                    (self.width - 288 + x - 892, w)
                } else {
                    (
                        self.main_x + (x - 244) * self.main_width / 620,
                        w * self.main_width / 620,
                    )
                };
                return (
                    nx,
                    self.footer_top + (24.0 * self.scale_y).round() as i32,
                    nw,
                    (h as f32 * self.scale_y).round() as i32,
                );
            }
            let half = (self.main_width - 16) / 2;
            return match x {
                244 => (self.main_x, self.footer_top + 20, self.main_width, h),
                434 => (self.main_x, self.footer_top + 74, half, h),
                608 => (self.main_x + half + 16, self.footer_top + 74, half, h),
                916 => (self.main_x, self.footer_top + 128, half, h),
                _ => (self.main_x + half + 16, self.footer_top + 128, half, h),
            };
        }
        if x < 216 {
            if x == 16 && (148..=418).contains(&y) && self.compact_nav {
                let index = (y - 148) / 54;
                let cell = (self.main_width - (self.nav_columns - 1) * 8) / self.nav_columns;
                return (
                    self.main_x + index % self.nav_columns * (cell + 8),
                    self.header_height + 20 + index / self.nav_columns * 54,
                    cell,
                    46,
                );
            }
            if self.compact_nav && y >= 86 {
                return (0, 0, 0, 0);
            }
            let ny = if y >= 615 {
                self.footer_top + self.y(y) - self.y(700)
            } else {
                self.y(y)
            };
            return (
                x,
                ny,
                w,
                if self.wide && y >= 86 {
                    (h as f32 * self.scale_y).round() as i32
                } else {
                    h
                },
            );
        }
        if y < 86 {
            if x >= 892 {
                return if !self.compact_nav {
                    (self.width - 264, y, 242, h)
                } else if self.header_height > 86 {
                    if self.width < 344 {
                        (24, 68, self.width - 48, 16)
                    } else {
                        (self.width - 205, 33, 181, h)
                    }
                } else {
                    (244, 60, self.width - 268, 18)
                };
            }
            return if self.header_height > 86 {
                (24, 86, self.width - 48, h)
            } else {
                (
                    self.main_x.max(244),
                    y,
                    self.width - self.main_x.max(244) - 24,
                    h,
                )
            };
        }
        if x >= 892 {
            return self.general(bounds);
        }
        if y == 700 {
            return (self.main_x, self.footer_top, self.main_width, h);
        }
        if y == 654 {
            return (
                self.main_x,
                self.footer_top
                    - if self.wide {
                        (46.0 * self.scale_y).round() as i32
                    } else {
                        48
                    },
                self.main_width,
                if self.wide {
                    (h as f32 * self.scale_y).round() as i32
                } else {
                    h
                },
            );
        }
        if y < 196 {
            if x == 738 {
                return (
                    self.main_x + self.main_width - 126,
                    if self.wide {
                        self.y(111)
                    } else {
                        self.page_top - 3
                    },
                    126,
                    if self.wide {
                        (38.0 * self.scale_y).round() as i32
                    } else {
                        38
                    },
                );
            }
            return (
                self.main_x,
                if self.wide {
                    self.y(y)
                } else {
                    self.page_top + y - 114
                },
                self.main_width,
                if y == 153 && self.single_column {
                    42
                } else if self.wide {
                    (h as f32 * self.scale_y).round() as i32
                } else {
                    h
                },
            );
        }
        if !self.single_column {
            return self.main(bounds);
        }
        match self.page {
            Page::Layout => {
                if y < 454 {
                    let right = x >= 564;
                    return self.card_column(
                        bounds,
                        if right { 564 } else { 244 },
                        if right { 256 } else { 0 },
                    );
                }
                if y < 540 {
                    let top = 710 + self.body_shift;
                    return if x == 244 {
                        (self.main_x, top, self.main_width, 106)
                    } else if x == 264 {
                        (self.main_x + 20, top + 16, self.main_width - 40, h)
                    } else {
                        (self.main_x + 20, top + 50, self.main_width - 40, h)
                    };
                }
                let top = 836 + self.body_shift;
                let half = (self.main_width - 52) / 2;
                if x == 244 {
                    (self.main_x, top, self.main_width, 152)
                } else if y == 555 {
                    (self.main_x + 20, top + 15, self.main_width - 40, h)
                } else if x == 264 {
                    (self.main_x + 20, top + 45, self.main_width - 40, h)
                } else {
                    (
                        self.main_x + 20 + if x == 692 { half + 12 } else { 0 },
                        top + 99,
                        half,
                        h,
                    )
                }
            }
            Page::Tyres => {
                if y < 326 {
                    if x == 244 {
                        return (self.main_x, y + self.body_shift, self.main_width, 148);
                    }
                    if y == 258 {
                        return (
                            self.main_x + 20,
                            250 + self.body_shift,
                            self.main_width - 40,
                            h,
                        );
                    }
                    if x == 504 {
                        return (
                            self.main_x + 20,
                            278 + self.body_shift,
                            self.main_width - 40,
                            h,
                        );
                    }
                    return self.main(bounds);
                }
                if y < 580 {
                    let right = x >= 564;
                    return self.card_column(
                        bounds,
                        if right { 564 } else { 244 },
                        38 + if right { 256 } else { 0 },
                    );
                }
                (
                    self.main_x + 20,
                    self.body_shift + if x < 564 { 874 } else { 928 },
                    self.main_width - 40,
                    h,
                )
            }
            Page::Colours => {
                let row = (y - 196) / 146;
                let right = x >= 564;
                self.card_column(
                    bounds,
                    if right { 564 } else { 244 },
                    row * 146 + if right { 146 } else { 0 },
                )
            }
            Page::Connection => {
                if y < 370 {
                    if x == 244 {
                        return (self.main_x, y + self.body_shift, self.main_width, 240);
                    }
                    let right = x >= 640;
                    return (
                        self.main_x + 20,
                        y + self.body_shift + if right { 82 } else { 0 },
                        self.main_width - 40,
                        h,
                    );
                }
                if (425..=545).contains(&y) && x == 660 {
                    return (
                        self.main_x + self.main_width - 100,
                        y + self.body_shift + 86,
                        80,
                        h,
                    );
                }
                if (425..=545).contains(&y) && x == 264 && h != 0 {
                    return (
                        self.main_x + 20,
                        y + self.body_shift + 86,
                        self.main_width - 130,
                        h,
                    );
                }
                let mapped = self.main(bounds);
                (
                    mapped.0,
                    mapped.1 + 86,
                    mapped.2,
                    if y == 628 { 44 } else { mapped.3 },
                )
            }
            Page::Shortcuts => {
                if (x == 264 || x == 274) && (251..=477).contains(&y) {
                    let ny = if y < 382 {
                        if y < 280 { 246 } else { 300 }
                    } else if y < 470 {
                        432
                    } else {
                        486
                    };
                    return (
                        self.main_x + 20 + if x == 274 { 10 } else { 0 },
                        ny + self.body_shift,
                        self.main_width - if x == 274 { 60 } else { 40 },
                        if y == 251 || y == 437 { 42 } else { h },
                    );
                }
                self.main(bounds)
            }
            Page::Diagnostics => {
                if x == 244 && y == 196 {
                    return (self.main_x, y + self.body_shift, self.main_width, 170);
                }
                if y == 252 {
                    return (
                        self.main_x + 20,
                        y + self.body_shift,
                        self.main_width - 40,
                        96,
                    );
                }
                let mapped = self.main(bounds);
                (
                    mapped.0,
                    mapped.1 + if y >= 354 { 32 } else { 0 },
                    mapped.2,
                    mapped.3,
                )
            }
        }
    }

    fn general(&self, (x, y, w, h): Bounds) -> Bounds {
        if self.wide {
            return (
                self.width - 288 + x - 892,
                self.y(y),
                w,
                (h as f32 * self.scale_y).round() as i32,
            );
        }
        let inner = self.main_width - 40;
        if self.general_height > 400 {
            return (
                self.main_x + 20 + (x - 916) * inner / 240,
                self.general_top + y - 86,
                w * inner / 240,
                h,
            );
        }
        if y < 180 {
            return (self.main_x + 20, self.general_top + y - 95, inner, h);
        }
        if y == 345 {
            return (0, 0, 0, 0);
        }
        if y == 556 {
            return (self.main_x + 20, self.general_top + 270, inner, h);
        }
        let col = (inner - 16) / 2;
        if y >= 578 {
            return if y < 611 {
                (self.main_x + 20, self.general_top + 288, inner, h)
            } else {
                (
                    self.main_x + 20 + if y >= 640 { col + 16 } else { 0 },
                    self.general_top + 321,
                    col,
                    h,
                )
            };
        }
        let behaviour = y >= 369;
        (
            self.main_x + 20 + if behaviour { col + 16 } else { 0 } + (x - 916) * col / 240,
            self.general_top + 84 + y - if behaviour { 369 } else { 180 },
            w * col / 240,
            h,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cards(page: Page) -> Vec<Bounds> {
        match page {
            Page::Layout => vec![
                (244, 196, 300, 238),
                (564, 196, 300, 238),
                (244, 454, 620, 66),
                (244, 540, 620, 100),
            ],
            Page::Tyres => vec![
                (244, 196, 620, 110),
                (244, 326, 300, 236),
                (564, 326, 300, 236),
            ],
            Page::Colours => (0..6)
                .map(|i| (244 + i % 2 * 320, 196 + i / 2 * 146, 300, 126))
                .collect(),
            Page::Connection => vec![(244, 196, 620, 154), (244, 370, 620, 240)],
            Page::Shortcuts => vec![(244, 196, 620, 166), (244, 382, 620, 166)],
            Page::Diagnostics => vec![(244, 196, 620, 138), (244, 354, 620, 286)],
        }
    }

    #[test]
    fn sections_fit_landscape_portrait_and_small_windows_without_overlaps() {
        for (width, height) in [
            (1920, 1000),
            (1280, 660),
            (1180, 780),
            (1080, 1800),
            (900, 1400),
            (720, 1100),
            (540, 960),
            (360, 640),
            (320, 480),
        ] {
            for page in Page::ALL {
                let layout = ResponsiveLayout::new(width, height, page);
                let mapped: Vec<_> = cards(page).into_iter().map(|b| layout.map(b)).collect();
                for &(x, y, w, h) in &mapped {
                    assert!(
                        x >= 0
                            && y >= 0
                            && w > 0
                            && h > 0
                            && x + w <= width
                            && y + h <= layout.content_height,
                        "{page:?} at {width}x{height}: card {:?} escaped the content",
                        (x, y, w, h)
                    );
                    if !layout.wide {
                        assert!(y + h < layout.general_top);
                    }
                }
                for (i, &(x, y, w, h)) in mapped.iter().enumerate() {
                    for &(rx, ry, rw, rh) in &mapped[i + 1..] {
                        assert!(
                            x + w <= rx || rx + rw <= x || y + h <= ry || ry + rh <= y,
                            "{page:?} cards overlap at {width}x{height}"
                        );
                    }
                }
                if !layout.wide {
                    assert!(layout.general_top + layout.general_height < layout.footer_top - 48);
                }
                for x in [244, 434, 608, 916, 22] {
                    let (x, y, w, h) = layout.map((x, 724, 174, 38));
                    assert!(x >= 0 && x + w <= width && y + h <= layout.content_height);
                }
            }
        }
    }

    #[test]
    fn portrait_moves_general_controls_below_settings_and_narrow_cards_stack() {
        let portrait = ResponsiveLayout::new(1080, 1800, Page::Layout);
        assert!(!portrait.wide);
        assert!(!portrait.compact_nav);
        assert!(portrait.map((916, 210, 114, 38)).1 > portrait.map((244, 540, 620, 100)).1 + 100);
        let narrow = ResponsiveLayout::new(540, 960, Page::Layout);
        assert!(narrow.compact_nav && narrow.single_column);
        let halo = narrow.map((244, 196, 300, 238));
        let tyres = narrow.map((564, 196, 300, 238));
        assert_eq!(halo.0, tyres.0);
        assert!(halo.1 + halo.3 < tyres.1);
        assert!(narrow.content_height > narrow.height);
    }
}
