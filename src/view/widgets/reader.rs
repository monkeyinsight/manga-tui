use once_cell::sync::Lazy;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget, Wrap};
use throbber_widgets_tui::{Throbber, ThrobberState};
use tui_widget_list::PreRender;

use crate::global::CURRENT_LIST_ITEM_STYLE;

pub static STYLE_PAGE_BOOKMARKED: Lazy<Style> = Lazy::new(|| Style::new().on_green().black());

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum PageItemState {
    Loading,
    FinishedLoad,
    FailedLoad,
    Waiting,
}

#[derive(Clone)]
pub struct PagesItem {
    pub number: usize,
    pub state: PageItemState,
    pub loading_state: ThrobberState,
    pub style: Style,
}

impl Widget for PagesItem {
    fn render(mut self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [chapter_number_area, loader_area] = layout.areas(area);

        let block_style = self.style;

        Block::default().style(block_style).render(area, buf);

        let page = Paragraph::new(format!("Page {}", self.number)).wrap(Wrap { trim: true });

        match self.state {
            PageItemState::Loading => {
                let loader = Throbber::default()
                    .label("Loading")
                    .style(Style::default().fg(Color::Yellow))
                    .throbber_set(throbber_widgets_tui::BRAILLE_SIX)
                    .use_type(throbber_widgets_tui::WhichUse::Spin);

                page.render(chapter_number_area, buf);

                StatefulWidget::render(loader, loader_area, buf, &mut self.loading_state);
            },
            PageItemState::FinishedLoad => {
                page.render(area, buf);
            },
            PageItemState::FailedLoad => {
                page.render(chapter_number_area, buf);
                Paragraph::new("⚠").wrap(Wrap { trim: true }).red().bold().render(loader_area, buf);
            },
            PageItemState::Waiting => {
                page.render(chapter_number_area, buf);
                Paragraph::new("💤").wrap(Wrap { trim: true }).bold().render(loader_area, buf);
            },
        }
    }
}

impl PreRender for PagesItem {
    fn pre_render(&mut self, context: &tui_widget_list::PreRenderContext) -> u16 {
        if context.is_selected {
            self.style = *CURRENT_LIST_ITEM_STYLE;
        }
        2
    }
}

impl PagesItem {
    pub fn new(number: usize) -> Self {
        Self {
            number,
            state: PageItemState::Waiting,
            loading_state: ThrobberState::default(),
            style: Style::default(),
        }
    }

    pub fn on_tick(&mut self) {
        if self.state == PageItemState::Loading {
            self.loading_state.calc_next();
        }
    }
}

#[derive(Clone, Default)]
pub struct PagesList {
    pub pages: Vec<PagesItem>,
}

impl PagesList {
    pub fn new(pages: Vec<PagesItem>) -> Self {
        Self { pages }
    }

    pub fn on_tick(&mut self) {
        for page in self.pages.iter_mut() {
            page.on_tick();
        }
    }

    fn reset_style(&mut self) {
        self.pages.iter_mut().for_each(|page| page.style = Style::default())
    }

    pub fn highlight_page_as_bookmarked(&mut self, page_index: usize) {
        self.reset_style();
        if let Some(page) = self.pages.get_mut(page_index) {
            page.style = *STYLE_PAGE_BOOKMARKED;
        }
    }
}

#[derive(Debug, Default)]
pub struct PagesListState {
    pub list_state: tui_widget_list::ListState,
    pub page_bookmarked: Option<usize>,
}

impl PagesListState {
    pub fn new(page_bookmarked: Option<usize>) -> Self {
        Self {
            list_state: tui_widget_list::ListState::default(),
            page_bookmarked,
        }
    }

    pub fn set_page_bookmarked(&mut self, page_bookmarked: usize) {
        self.page_bookmarked = Some(page_bookmarked);
    }
}

impl StatefulWidget for PagesList {
    type State = PagesListState;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(page) = state.page_bookmarked {
            if state.list_state.selected.is_none() {
                state.list_state.select(Some(page));
            }
            self.highlight_page_as_bookmarked(page);
        }

        let items = tui_widget_list::List::new(self.pages);

        StatefulWidget::render(items, area, buf, &mut state.list_state)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_highlights_page_item_which_is_bookmarked() {
        let mut page_list = PagesList::new(vec![PagesItem::new(0), PagesItem::new(1)]);

        page_list.highlight_page_as_bookmarked(1);

        let page_item = page_list.pages[1].clone();

        assert_eq!(*STYLE_PAGE_BOOKMARKED, page_item.style);
    }

    #[test]
    fn it_keeps_only_one_item_bookmarked_at_a_time() {
        let mut page1 = PagesItem::new(0);

        page1.style = *STYLE_PAGE_BOOKMARKED;

        let mut page_list = PagesList::new(vec![page1, PagesItem::new(1)]);

        page_list.highlight_page_as_bookmarked(1);

        let page_item_with_no_highlight = page_list.pages[0].clone();
        let page_item_highlighet = page_list.pages[1].clone();

        assert_eq!(*STYLE_PAGE_BOOKMARKED, page_item_highlighet.style);
        assert_eq!(Style::default(), page_item_with_no_highlight.style);
    }
}
