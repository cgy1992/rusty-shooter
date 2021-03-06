//! Contains all helper functions that creates styled widgets for game user interface.
//! However most of the styles are used from dark theme of rg3d-ui library so there
//! is not much.

use crate::{UINodeHandle, BuildContext};
use rg3d::{
    engine::resource_manager::ResourceManager,
    utils,
    core::color::Color,
    resource::texture::TextureKind,
    gui::{
        brush::Brush,
        scroll_bar::ScrollBarBuilder,
        check_box::CheckBoxBuilder,
        VerticalAlignment,
        HorizontalAlignment,
        widget::WidgetBuilder,
        image::ImageBuilder,
        Thickness,
        Orientation,
        scroll_viewer::ScrollViewerBuilder,
    },
};

pub struct ScrollBarData {
    pub min: f32,
    pub max: f32,
    pub value: f32,
    pub step: f32,
    pub row: usize,
    pub column: usize,
    pub margin: Thickness,
    pub show_value: bool,
    pub orientation: Orientation,
}

pub fn create_scroll_bar(ctx: &mut BuildContext, resource_manager: &mut ResourceManager, data: ScrollBarData) -> UINodeHandle {
    let mut wb = WidgetBuilder::new();
    match data.orientation {
        Orientation::Vertical => wb = wb.with_width(30.0),
        Orientation::Horizontal => wb = wb.with_height(30.0),
    }
    ScrollBarBuilder::new(wb
        .on_row(data.row)
        .on_column(data.column)
        .with_margin(data.margin))
        .with_orientation(data.orientation)
        .show_value(data.show_value)
        .with_max(data.max)
        .with_min(data.min)
        .with_step(data.step)
        .with_value(data.value)
        .with_indicator(ImageBuilder::new(WidgetBuilder::new()
            .with_background(Brush::Solid(Color::opaque(60, 60, 60))))
            .with_opt_texture(utils::into_any_arc(resource_manager.request_texture("data/ui/circle.png", TextureKind::RGBA8)))
            .build(ctx))
        .build(ctx)
}

pub fn create_check_box(ctx: &mut BuildContext, resource_manager: &mut ResourceManager, row: usize, column: usize, checked: bool) -> UINodeHandle {
    CheckBoxBuilder::new(WidgetBuilder::new()
        .with_margin(Thickness::uniform(2.0))
        .with_width(24.0)
        .with_height(24.0)
        .on_row(row)
        .on_column(column)
        .with_vertical_alignment(VerticalAlignment::Center)
        .with_horizontal_alignment(HorizontalAlignment::Left))
        .checked(Some(checked))
        .with_check_mark(ImageBuilder::new(WidgetBuilder::new())
            .with_opt_texture(utils::into_any_arc(resource_manager.request_texture("data/ui/check_mark.png", TextureKind::RGBA8)))
            .build(ctx))
        .build(ctx)
}

pub fn create_scroll_viewer(ctx: &mut BuildContext, resource_manager: &mut ResourceManager) -> UINodeHandle {
    ScrollViewerBuilder::new(WidgetBuilder::new())
        .with_horizontal_scroll_bar(create_scroll_bar(ctx, resource_manager, ScrollBarData{
            min: 0.0,
            max: 0.0,
            value: 0.0,
            step: 0.0,
            row: 0,
            column: 0,
            margin: Default::default(),
            show_value: false,
            orientation: Orientation::Horizontal
        }))
        .with_vertical_scroll_bar(create_scroll_bar(ctx, resource_manager, ScrollBarData{
            min: 0.0,
            max: 0.0,
            value: 0.0,
            step: 0.0,
            row: 0,
            column: 0,
            margin: Default::default(),
            show_value: false,
            orientation: Orientation::Vertical
        }))
        .build(ctx)
}