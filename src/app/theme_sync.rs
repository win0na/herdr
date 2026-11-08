use super::App;

impl App {
    pub(crate) fn set_host_terminal_appearance_state(
        &mut self,
        appearance: Option<crate::terminal_theme::HostAppearance>,
        explicit: bool,
    ) -> bool {
        if self.state.host_terminal_appearance == appearance
            && self.state.host_terminal_appearance_explicit == explicit
        {
            return false;
        }
        self.state.host_terminal_appearance = appearance;
        self.state.host_terminal_appearance_explicit = explicit;
        self.apply_host_terminal_appearance_to_panes();
        self.refresh_effective_app_theme()
    }

    pub(crate) fn set_client_host_terminal_theme(
        &mut self,
        theme: crate::terminal_theme::TerminalTheme,
    ) -> bool {
        self.state.client_host_terminal_theme = theme;
        self.recompute_host_terminal_theme()
    }

    pub(crate) fn set_fallback_host_terminal_theme(
        &mut self,
        theme: crate::terminal_theme::TerminalTheme,
    ) -> bool {
        self.state.fallback_host_terminal_theme = theme;
        self.recompute_host_terminal_theme()
    }

    fn recompute_host_terminal_theme(&mut self) -> bool {
        let theme = merge_terminal_themes(
            self.state.client_host_terminal_theme,
            self.state.fallback_host_terminal_theme,
        );
        self.set_host_terminal_theme(theme)
    }

    fn set_host_terminal_theme(&mut self, theme: crate::terminal_theme::TerminalTheme) -> bool {
        if theme == self.state.host_terminal_theme {
            return false;
        }
        self.state.host_terminal_theme = theme;
        self.apply_host_terminal_theme_to_panes();
        true
    }

    pub(super) fn refresh_effective_app_theme(&mut self) -> bool {
        let (palette, theme_name) = super::resolve_effective_theme(
            &self.state.theme_runtime,
            self.state.host_terminal_appearance,
        );
        if self.state.theme_name == theme_name && self.state.palette == palette {
            return false;
        }
        self.state.theme_name = theme_name;
        self.state.palette = palette;
        self.render_dirty.request_generic();
        self.render_notify.notify_one();
        true
    }

    fn apply_host_terminal_appearance_to_panes(&self) {
        for runtime in self.terminal_runtimes.values() {
            runtime.apply_host_terminal_appearance(self.state.host_terminal_appearance);
        }
    }

    pub(crate) fn apply_host_terminal_theme_to_panes(&self) {
        for runtime in self.terminal_runtimes.values() {
            runtime.apply_host_terminal_theme(self.state.host_terminal_theme);
        }

        self.render_dirty.request_generic();
        self.render_notify.notify_one();
    }
}

/// Client colours win; the configured palette fills anything the client did
/// not report. A control stream reports nothing, so it gets the whole
/// fallback theme.
fn merge_terminal_themes(
    client: crate::terminal_theme::TerminalTheme,
    fallback: crate::terminal_theme::TerminalTheme,
) -> crate::terminal_theme::TerminalTheme {
    let mut palette = [None; 256];
    for (index, color) in palette.iter_mut().enumerate() {
        *color = client.palette[index].or(fallback.palette[index]);
    }
    crate::terminal_theme::TerminalTheme {
        foreground: client.foreground.or(fallback.foreground),
        background: client.background.or(fallback.background),
        palette,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal_theme::{RgbColor, TerminalTheme};

    fn color(r: u8, g: u8, b: u8) -> Option<RgbColor> {
        Some(RgbColor { r, g, b })
    }

    #[test]
    fn client_colors_override_the_configured_palette() {
        let client = TerminalTheme {
            foreground: color(1, 2, 3),
            background: None,
            palette: [None; 256],
        };
        let fallback = TerminalTheme {
            foreground: color(9, 9, 9),
            background: color(4, 5, 6),
            palette: [color(7, 8, 9); 256],
        };

        let merged = merge_terminal_themes(client, fallback);
        assert_eq!(merged.foreground, color(1, 2, 3));
        assert_eq!(merged.background, color(4, 5, 6));
        assert_eq!(merged.palette[0], color(7, 8, 9));
    }

    #[test]
    fn an_empty_client_theme_uses_the_fallback() {
        let fallback = TerminalTheme {
            foreground: color(9, 9, 9),
            background: color(4, 5, 6),
            palette: [color(7, 8, 9); 256],
        };

        assert_eq!(
            merge_terminal_themes(TerminalTheme::default(), fallback),
            fallback
        );
    }
}
