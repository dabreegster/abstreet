use crate::{ButtonBuilder, EventCtx, Panel, Widget};

struct Tab {
    tab_id: String,
    bar_item: ButtonBuilder<'static, 'static>,
    content: Widget,
}

impl Tab {
    fn new(tab_id: String, bar_item: ButtonBuilder<'static, 'static>, content: Widget) -> Self {
        Self {
            tab_id,
            bar_item,
            content,
        }
    }

    fn build_bar_item_widget(&self, ctx: &EventCtx, active: bool) -> Widget {
        self.bar_item
            .clone()
            .disabled(active)
            .build_widget(ctx, &self.tab_id)
    }
}

pub struct TabController {
    id: String,
    tabs: Vec<Tab>,
    active_child: usize,
}

impl TabController {
    pub fn new(
        id: String,
        initial_bar_item: ButtonBuilder<'static, 'static>,
        initial_content: Widget,
    ) -> Self {
        let mut tc = Self {
            id,
            tabs: vec![],
            active_child: 0,
        };
        tc.push_tab(initial_bar_item, initial_content);

        tc
    }

    /// Add a new tab.
    ///
    /// `bar_item`: The button shown in the tab bar
    /// `content`: The content shown when this tab's `bar_item` is clicked
    pub fn push_tab(&mut self, bar_item: ButtonBuilder<'static, 'static>, content: Widget) {
        let tab_id = self.tab_id(self.tabs.len() + 1);
        let tab = Tab::new(tab_id, bar_item, content);
        self.tabs.push(tab);
    }

    /// A widget containing the tab bar and a content pane with the currently active tab.
    pub fn build_widget(&mut self, ctx: &EventCtx) -> Widget {
        Widget::col(vec![
            self.build_bar_items(ctx),
            self.pop_active_content()
                .container()
                .padding(16)
                .bg(ctx.style().section_bg)
                .named(self.active_content_id()),
        ])
    }

    pub fn handle_action(&mut self, ctx: &EventCtx, action: &str, panel: &mut Panel) -> bool {
        if !action.starts_with(&self.id) {
            return false;
        }

        let tab_idx = self
            .tabs
            .iter()
            .enumerate()
            .find(|(_idx, tab)| &tab.tab_id == action)
            .expect(&format!("invalid tab id: {}", action))
            .0;
        self.activate_tab(ctx, tab_idx, panel);
        true
    }

    fn active_content_id(&self) -> String {
        format!("{}_active_content", self.id)
    }

    fn bar_items_id(&self) -> String {
        format!("{}_bar_items", self.id)
    }

    fn tab_id(&self, tab_index: usize) -> String {
        format!("{}_tab_{}", self.id, tab_index)
    }

    fn pop_active_content(&mut self) -> Widget {
        let mut tmp = Widget::nothing();
        std::mem::swap(&mut self.tabs[self.active_child].content, &mut tmp);
        tmp
    }

    fn build_bar_items(&self, ctx: &EventCtx) -> Widget {
        let bar_items = self
            .tabs
            .iter()
            .enumerate()
            .map(|(idx, tab)| tab.build_bar_item_widget(ctx, idx == self.active_child))
            .collect();
        Widget::row(bar_items)
            .container()
            .bg(ctx.style().section_bg)
            .named(self.bar_items_id())
    }

    fn activate_tab(&mut self, ctx: &EventCtx, tab_idx: usize, panel: &mut Panel) {
        let old_idx = self.active_child;
        self.active_child = tab_idx;

        let mut bar_items = self.build_bar_items(ctx);
        panel.swap_container_content(&self.bar_items_id(), &mut bar_items);

        let mut content = self.pop_active_content();
        panel.swap_container_content(&self.active_content_id(), &mut content);
        self.tabs[old_idx].content = content;
    }
}