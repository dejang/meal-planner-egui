use egui::TextStyle;

pub trait TextStyleExt {
    fn heading1() -> Self;
    fn heading2() -> Self;
    fn heading3() -> Self;
    fn small() -> Self;
}

impl TextStyleExt for TextStyle {
    fn heading1() -> Self {
        TextStyle::Heading
    }

    fn heading2() -> Self {
        TextStyle::Body
    }

    fn heading3() -> Self {
        TextStyle::Body
    }

    fn small() -> Self {
        TextStyle::Small
    }
}


