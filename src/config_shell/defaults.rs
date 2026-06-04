use crate::config_shell::theme::MaterialScheme;

pub fn default_light_scheme() -> MaterialScheme {
    MaterialScheme {
        primary: [68, 94, 145],
        surface_tint: [68, 94, 145],
        on_primary: [255, 255, 255],
        primary_container: [216, 226, 255],
        on_primary_container: [0, 26, 66],
        secondary: [87, 94, 113],
        on_secondary: [255, 255, 255],
        secondary_container: [219, 226, 249],
        on_secondary_container: [20, 27, 44],
        tertiary: [113, 85, 115],
        on_tertiary: [255, 255, 255],
        tertiary_container: [252, 215, 251],
        on_tertiary_container: [41, 19, 45],
        error: [186, 26, 26],
        on_error: [255, 255, 255],
        error_container: [255, 218, 214],
        on_error_container: [65, 0, 2],
        background: [249, 249, 255],
        on_background: [26, 27, 32],
        surface: [249, 249, 255],
        on_surface: [26, 27, 32],
        surface_variant: [225, 226, 236],
        on_surface_variant: [68, 71, 79],
        outline: [117, 119, 127],
        outline_variant: [196, 198, 208],
    }
}

// -----------------------------
// DEFAULT DARK SCHEME
// -----------------------------
pub fn default_dark_scheme() -> MaterialScheme {
    MaterialScheme {
        primary: [173, 198, 255],
        surface_tint: [173, 198, 255],
        on_primary: [17, 47, 96],
        primary_container: [43, 70, 120],
        on_primary_container: [216, 226, 255],
        secondary: [191, 198, 220],
        on_secondary: [41, 48, 65],
        secondary_container: [63, 71, 89],
        on_secondary_container: [219, 226, 249],
        tertiary: [222, 188, 223],
        on_tertiary: [64, 40, 67],
        tertiary_container: [88, 62, 91],
        on_tertiary_container: [252, 215, 251],
        error: [255, 180, 171],
        on_error: [105, 0, 5],
        error_container: [147, 0, 10],
        on_error_container: [255, 218, 214],
        background: [17, 19, 24],
        on_background: [226, 226, 233],
        surface: [17, 19, 24],
        on_surface: [226, 226, 233],
        surface_variant: [68, 71, 79],
        on_surface_variant: [196, 198, 208],
        outline: [142, 144, 153],
        outline_variant: [68, 71, 79],
    }
}
