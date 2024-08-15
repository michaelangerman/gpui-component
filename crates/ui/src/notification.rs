use std::{sync::Arc, time::Duration};

use gpui::{
    div, prelude::FluentBuilder as _, px, Animation, AnimationExt, ClickEvent, DismissEvent,
    ElementId, EventEmitter, InteractiveElement as _, IntoElement, ParentElement as _, Render,
    SharedString, StatefulInteractiveElement, Styled, View, ViewContext, VisualContext,
    WindowContext,
};
use smol::Timer;

use crate::{
    button::Button, h_flex, theme::ActiveTheme as _, v_flex, Icon, IconName, Sizable as _,
    StyledExt,
};

pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

pub struct Notification {
    /// The id is used make the notification unique.
    /// Then you push a notification with the same id, the previous notification will be replaced.
    ///
    /// None means the notification will be added to the end of the list.
    id: ElementId,
    type_: NotificationType,
    title: Option<SharedString>,
    content: SharedString,
    icon: Option<Icon>,
    autohide: bool,
    on_click: Option<Arc<dyn Fn(&ClickEvent, &mut WindowContext)>>,
}

impl From<SharedString> for Notification {
    fn from(s: SharedString) -> Self {
        Self::new(s)
    }
}

impl From<&'static str> for Notification {
    fn from(s: &'static str) -> Self {
        Self::new(s)
    }
}

impl From<(NotificationType, &'static str)> for Notification {
    fn from((type_, content): (NotificationType, &'static str)) -> Self {
        Self::new(content).with_type(type_)
    }
}

impl From<(NotificationType, SharedString)> for Notification {
    fn from((type_, content): (NotificationType, SharedString)) -> Self {
        Self::new(content).with_type(type_)
    }
}

impl Notification {
    /// Create a new notification with the given content.
    ///
    /// default width is 320px.
    pub fn new(content: impl Into<SharedString>) -> Self {
        let id = uuid::Uuid::new_v4().to_string();

        Self {
            id: SharedString::from(id).into(),
            title: None,
            content: content.into(),
            type_: NotificationType::Info,
            icon: None,
            autohide: true,
            on_click: None,
        }
    }

    pub fn with_id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// Set the title of the notification, default is None.
    ///
    /// If tilte is None, the notification will not have a title.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the icon of the notification.
    ///
    /// If icon is None, the notification will use the default icon of the type.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn with_type(mut self, type_: NotificationType) -> Self {
        self.type_ = type_;
        self
    }

    pub fn info(mut self) -> Self {
        self.type_ = NotificationType::Info;
        self
    }

    pub fn success(mut self) -> Self {
        self.type_ = NotificationType::Success;
        self
    }

    pub fn warning(mut self) -> Self {
        self.type_ = NotificationType::Warning;
        self
    }

    pub fn error(mut self) -> Self {
        self.type_ = NotificationType::Error;
        self
    }

    /// Set the auto hide of the notification, default is true.
    pub fn autohide(mut self, autohide: bool) -> Self {
        self.autohide = autohide;
        self
    }

    /// Set the click callback of the notification.
    pub fn on_click(
        mut self,
        on_click: impl Fn(&ClickEvent, &mut WindowContext) + 'static,
    ) -> Self {
        self.on_click = Some(Arc::new(on_click));
        self
    }

    fn perform_autohide(&self, cx: &mut ViewContext<Self>) {
        if !self.autohide {
            return;
        }

        // Sleep for 5 seconds to autohide the notification
        cx.spawn(|view, mut cx| async move {
            Timer::after(Duration::from_secs(5)).await;
            let _ = view.update(&mut cx, |_, cx| cx.emit(DismissEvent));
        })
        .detach();
    }

    fn dismiss(&mut self, _: &ClickEvent, cx: &mut ViewContext<Self>) {
        cx.emit(DismissEvent);
    }
}
impl EventEmitter<DismissEvent> for Notification {}

impl Render for Notification {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let group_id = "notification-group";
        self.perform_autohide(cx);

        let icon = match self.icon.clone() {
            Some(icon) => icon,
            None => match self.type_ {
                NotificationType::Info => Icon::new(IconName::Info).text_color(crate::blue_500()),
                NotificationType::Success => {
                    Icon::new(IconName::CircleCheck).text_color(crate::green_500())
                }
                NotificationType::Warning => {
                    Icon::new(IconName::TriangleAlert).text_color(crate::yellow_500())
                }
                NotificationType::Error => {
                    Icon::new(IconName::CircleX).text_color(crate::red_500())
                }
            },
        };

        div()
            .w_96()
            .id("notification")
            .occlude()
            .group(group_id)
            .relative()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().popover)
            .rounded_md()
            .shadow_md()
            .py_2()
            .px_4()
            .gap_3()
            .child(
                div()
                    .absolute()
                    .map(|this| match self.title.is_some() {
                        true => this.top_3().left_4(),
                        false => this.top_2p5().left_4(),
                    })
                    .child(icon),
            )
            .child(
                v_flex()
                    .pl_6()
                    .gap_1()
                    .when_some(self.title.clone(), |this, title| {
                        this.child(div().text_sm().font_semibold().child(title))
                    })
                    .overflow_hidden()
                    .child(div().text_sm().child(self.content.clone())),
            )
            .when_some(self.on_click.clone(), |this, on_click| {
                this.cursor_pointer()
                    .on_click(cx.listener(move |_, event, cx| {
                        cx.emit(DismissEvent);
                        on_click(event, cx);
                    }))
            })
            .when(!self.autohide, |this| {
                this.child(
                    h_flex()
                        .absolute()
                        .top_1()
                        .right_1()
                        .invisible()
                        .group_hover(group_id, |this| this.visible())
                        .child(
                            Button::new("close", cx)
                                .icon(IconName::Close)
                                .ghost()
                                .xsmall()
                                .on_click(cx.listener(Self::dismiss)),
                        ),
                )
            })
            .with_animation(
                "slide-left",
                Animation::new(Duration::from_secs_f64(0.1)),
                move |this, delta| {
                    let x_offset = px(120.) + delta * px(-120.);
                    this.left(px(0.) + x_offset)
                },
            )
    }
}

/// A list of notifications.
pub struct NotificationList {
    notifications: Vec<View<Notification>>,
}

impl NotificationList {
    pub fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self {
            notifications: Vec::new(),
        }
    }

    pub fn push(&mut self, notification: impl Into<Notification>, cx: &mut ViewContext<Self>) {
        let notification = notification.into();
        let id = notification.id.clone();

        // Remove the notification by id, for keep unique.
        self.notifications.retain(|note| note.read(cx).id != id);

        let notification = cx.new_view(|_| notification);
        cx.subscribe(&notification, move |view, _, _: &DismissEvent, cx| {
            view.notifications.retain(|note| id != note.read(cx).id);
        })
        .detach();

        self.notifications.push(notification);
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut ViewContext<Self>) {
        self.notifications.clear();
        cx.notify();
    }
}

impl Render for NotificationList {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        let size = cx.viewport_size();

        let last_10_notes = self
            .notifications
            .iter()
            .rev()
            .take(10)
            .rev()
            .cloned()
            .collect::<Vec<_>>();

        div()
            .absolute()
            .top_4()
            .bottom_4()
            .right_4()
            .justify_end()
            .child(
                v_flex()
                    .absolute()
                    .right_0()
                    .h(size.height)
                    .gap_3()
                    .children(last_10_notes),
            )
    }
}
