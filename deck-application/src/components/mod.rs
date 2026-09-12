pub mod greeter;

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qfont.h");
        type QFont = cxx_qt_lib::QFont;

        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("include/qfont_features.h");
        pub fn set_qfont_feature(font: Pin<&mut QFont>, tag: &QString, value: u32);
    }
}
