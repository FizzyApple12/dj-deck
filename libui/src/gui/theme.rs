use masonry::{
    core::{DefaultProperties, StyleProperty, StyleSet},
    parley::{FontFamily, LineHeight},
    peniko::Color,
    properties::{
        ActiveBackground, Background, BarColor, BorderColor, BorderWidth, CaretColor,
        CheckmarkColor, CheckmarkStrokeWidth, ContentColor, CornerRadius, DisabledBackground,
        DisabledCheckmarkColor, DisabledContentColor, HoveredBorderColor, Padding,
        PlaceholderColor, SelectionColor, UnfocusedSelectionColor, types::Length,
    },
    widgets::{Button, Checkbox, Label, ProgressBar, Spinner, TextArea, TextInput},
};

pub const BORDER_WIDTH: f64 = 1.;

// Neutral color variations from https://v3.tailwindcss.com/docs/colors
pub const NEUTRAL_900: Color = Color::from_rgb8(0x17, 0x17, 0x17);
pub const NEUTRAL_800: Color = Color::from_rgb8(0x26, 0x26, 0x26);
pub const NEUTRAL_700: Color = Color::from_rgb8(0x40, 0x40, 0x40);
pub const NEUTRAL_600: Color = Color::from_rgb8(0x52, 0x52, 0x52);
pub const NEUTRAL_500: Color = Color::from_rgb8(0x73, 0x73, 0x73);
pub const NEUTRAL_400: Color = Color::from_rgb8(0xa3, 0xa3, 0xa3);
pub const NEUTRAL_300: Color = Color::from_rgb8(0xd4, 0xd4, 0xd4);
pub const NEUTRAL_200: Color = Color::from_rgb8(0xe4, 0xe4, 0xe4);
pub const NEUTRAL_100: Color = Color::from_rgb8(0xfa, 0xfa, 0xfa);

pub const ACCENT_COLOR: Color = Color::from_rgb8(0x3b, 0x7e, 0xe4);
pub const TEXT_COLOR: Color = NEUTRAL_100;
pub const DISABLED_TEXT_COLOR: Color = NEUTRAL_400;
const PLACEHOLDER_COLOR: Color = Color::from_rgba8(0xff, 0x00, 0xff, 0x8f);

// TODO: The following constants are not being used in properties
pub const TEXT_SIZE_NORMAL: f32 = 15.0;
pub const BASIC_WIDGET_HEIGHT: f64 = 18.0;
pub const BORDERED_WIDGET_HEIGHT: f64 = 24.0;
pub const SCROLLBAR_WIDTH: f64 = 8.;
pub const SCROLLBAR_PAD: f64 = 2.;
pub const SCROLLBAR_MIN_SIZE: f64 = 45.;
pub const SCROLLBAR_RADIUS: f64 = 5.;
pub const SCROLLBAR_EDGE_WIDTH: f64 = 1.;
pub const DEFAULT_GAP: Length = Length::const_px(10.0);
pub const DEFAULT_SPACER_LEN: Length = Length::const_px(10.0);
pub const WIDGET_CONTROL_COMPONENT_PADDING: f64 = 4.0;

pub fn default_property_set() -> DefaultProperties {
    let mut properties = DefaultProperties::new();

    // Button
    properties.insert::<Button, _>(Padding::from_vh(6., 16.));
    properties.insert::<Button, _>(CornerRadius { radius: 6. });
    properties.insert::<Button, _>(BorderWidth {
        width: BORDER_WIDTH,
    });

    properties.insert::<Button, _>(Background::Color(NEUTRAL_800));
    properties.insert::<Button, _>(ActiveBackground(Background::Color(NEUTRAL_700)));
    properties.insert::<Button, _>(DisabledBackground(Background::Color(Color::BLACK)));
    properties.insert::<Button, _>(BorderColor { color: NEUTRAL_700 });
    properties.insert::<Button, _>(HoveredBorderColor(BorderColor { color: NEUTRAL_500 }));

    // Checkbox
    properties.insert::<Checkbox, _>(CornerRadius { radius: 4. });
    properties.insert::<Checkbox, _>(BorderWidth {
        width: BORDER_WIDTH,
    });

    properties.insert::<Checkbox, _>(Background::Color(NEUTRAL_800));
    properties.insert::<Checkbox, _>(ActiveBackground(Background::Color(NEUTRAL_700)));
    properties.insert::<Checkbox, _>(DisabledBackground(Background::Color(Color::BLACK)));
    properties.insert::<Checkbox, _>(BorderColor { color: NEUTRAL_700 });
    properties.insert::<Checkbox, _>(HoveredBorderColor(BorderColor { color: NEUTRAL_500 }));

    properties.insert::<Checkbox, _>(CheckmarkStrokeWidth { width: 2.0 });
    properties.insert::<Checkbox, _>(CheckmarkColor { color: TEXT_COLOR });
    properties.insert::<Checkbox, _>(DisabledCheckmarkColor(CheckmarkColor {
        color: DISABLED_TEXT_COLOR,
    }));

    // TextInput
    properties.insert::<TextInput, _>(Padding::from_vh(6., 12.));
    properties.insert::<TextInput, _>(CornerRadius { radius: 4. });
    properties.insert::<TextInput, _>(BorderWidth {
        width: BORDER_WIDTH,
    });
    properties.insert::<TextInput, _>(BorderColor { color: NEUTRAL_600 });
    properties.insert::<TextInput, _>(PlaceholderColor::new(PLACEHOLDER_COLOR));
    properties.insert::<TextInput, _>(CaretColor { color: TEXT_COLOR });
    properties.insert::<TextInput, _>(SelectionColor {
        color: ACCENT_COLOR,
    });
    properties.insert::<TextInput, _>(UnfocusedSelectionColor(SelectionColor {
        color: DISABLED_TEXT_COLOR,
    }));

    // TextArea
    properties.insert::<TextArea<false>, _>(ContentColor::new(TEXT_COLOR));
    properties
        .insert::<TextArea<false>, _>(DisabledContentColor(ContentColor::new(DISABLED_TEXT_COLOR)));
    properties.insert::<TextArea<false>, _>(CaretColor { color: TEXT_COLOR });
    properties.insert::<TextArea<false>, _>(SelectionColor {
        color: ACCENT_COLOR,
    });
    properties.insert::<TextArea<false>, _>(UnfocusedSelectionColor(SelectionColor {
        color: DISABLED_TEXT_COLOR,
    }));
    properties.insert::<TextArea<true>, _>(ContentColor::new(TEXT_COLOR));
    properties
        .insert::<TextArea<true>, _>(DisabledContentColor(ContentColor::new(DISABLED_TEXT_COLOR)));
    properties.insert::<TextArea<true>, _>(CaretColor { color: TEXT_COLOR });
    properties.insert::<TextArea<true>, _>(SelectionColor {
        color: ACCENT_COLOR,
    });
    properties.insert::<TextArea<true>, _>(UnfocusedSelectionColor(SelectionColor {
        color: DISABLED_TEXT_COLOR,
    }));

    // Label
    properties.insert::<Label, _>(Padding::from_vh(0., 2.));
    properties.insert::<Label, _>(ContentColor::new(TEXT_COLOR));
    properties.insert::<Label, _>(DisabledContentColor(ContentColor::new(DISABLED_TEXT_COLOR)));

    // ProgressBar
    properties.insert::<ProgressBar, _>(CornerRadius { radius: 2. });
    properties.insert::<ProgressBar, _>(BorderWidth {
        width: BORDER_WIDTH,
    });

    properties.insert::<ProgressBar, _>(Background::Color(NEUTRAL_900));
    properties.insert::<ProgressBar, _>(BorderColor { color: NEUTRAL_800 });
    properties.insert::<ProgressBar, _>(BarColor(ACCENT_COLOR));

    // Spinner
    properties.insert::<Spinner, _>(ContentColor::new(TEXT_COLOR));

    properties
}

pub fn default_text_styles(styles: &mut StyleSet) {
    styles.insert(StyleProperty::LineHeight(LineHeight::FontSizeRelative(1.2)));
    styles.insert(FontFamily::parse("helvetica").unwrap().into());
}
